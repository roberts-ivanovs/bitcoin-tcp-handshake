use std::net::SocketAddr;

use bitcoin::network::constants::{self, ServiceFlags};
use bitcoin::network::message::{NetworkMessage, RawNetworkMessage};
use bitcoin::network::message_network::VersionMessage;
use bitcoin::network::Address;
use models::{BitcoinMessage, Verack, Version};
use rand::Rng;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::WriteHalf;
use tokio::net::TcpStream;
use tracing::instrument;

use super::actor::ConnectionActor;
use crate::error::{self, Error};

#[derive(Debug)]
pub struct ConnectionHandle {
    to_actor_sender: tokio::sync::mpsc::Sender<ToConnectionHandle>,
    from_actor_receiver: tokio::sync::mpsc::Receiver<FromConnectionHandle>,
    network: constants::Network,
    #[allow(dead_code)]
    actor_handle: tokio::task::JoinHandle<()>,
    peer_address: SocketAddr,
    sender_address: SocketAddr,
}

#[derive(Debug)]
pub enum ToConnectionHandle {
    ToBitcoinNode(RawNetworkMessage),
}

#[derive(Debug)]
pub enum FromConnectionHandle {
    /// Message from the bitcoin node. We are not interested in the magic number,
    /// therefore we are using NetworkMessage instead of RawNetworkMessage.
    FromBitcoinNode(NetworkMessage),
}

impl ConnectionHandle {
    #[instrument(err, ret)]
    pub async fn new(peer_address: SocketAddr, sender_address: SocketAddr, network: constants::Network) -> Result<Self, Error> {
        // Init communication primitives with the actor
        let (to_actor_sender, to_actor_receiver) = tokio::sync::mpsc::channel(10);
        let (from_actor_sender, from_actor_receiver) = tokio::sync::mpsc::channel(10);

        // Spawn the actor
        let actor = ConnectionActor::new(peer_address, to_actor_receiver, from_actor_sender)?;
        let actor_handle = tokio::spawn(async move {
            actor.run().await;
        });
        Ok(Self {
            network,
            peer_address,
            sender_address,
            actor_handle,
            to_actor_sender,
            from_actor_receiver,
        })
    }

    /// Common send method for all messages, will properly map the error types
    #[instrument(skip(self), err, ret)]
    async fn send_to_node(&self, message: NetworkMessage) -> Result<(), Error> {
        let message = RawNetworkMessage {
            magic: self.network.magic(),
            payload: message,
        };
        let message = ToConnectionHandle::ToBitcoinNode(message);
        self.to_actor_sender
            .send(message)
            .await
            .map_err(|_| Error::ActorSendError)?;
        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub async fn receive(&mut self) -> Result<FromConnectionHandle, Error> {
        self.from_actor_receiver
            .recv()
            .await
            .ok_or(Error::ConnectionDied)
    }

    pub async fn send_version(&self) -> Result<(), Error> {
        let message = build_version_message(&self.peer_address, &self.sender_address);
        let message = NetworkMessage::Version(message);

        self.send_to_node(message).await
    }

    pub async fn receive_version(&mut self) -> Result<VersionMessage, Error> {
        let message = self.receive().await?;

        let FromConnectionHandle::FromBitcoinNode(NetworkMessage::Version(message)) = message else {
            return Err(Error::UnexpectedConnectionMessage(message))
        };

        Ok(message)
    }

    pub async fn send_verack(&self) -> Result<(), Error> {
        let message = NetworkMessage::Verack;

        self.send_to_node(message).await
    }

    pub async fn receive_verack(&mut self) -> Result<(), Error> {
        let message = self.receive().await?;
        let FromConnectionHandle::FromBitcoinNode(NetworkMessage::Verack) = message else {
            return Err(Error::UnexpectedConnectionMessage(message))
        };

        Ok(())
    }
}

fn build_version_message(peer_address: &SocketAddr, sender_address: &SocketAddr) -> VersionMessage {
    /// The height of the block that the node is currently at.
    /// We are always at the genesis block. because our implementation is not a real node.
    const START_HEIGHT: i32 = 0;
    const USER_AGENT: &str = "bitcoin-handshake";
    const SERVICES: ServiceFlags = ServiceFlags::NONE;

    let sender = Address::new(sender_address, SERVICES);
    let timestamp = chrono::Utc::now().timestamp();
    let receiver = Address::new(peer_address, SERVICES);
    let nonce = rand::thread_rng().gen();
    let user_agent = USER_AGENT.to_string();

    // Construct the message
    VersionMessage::new(
        SERVICES,
        timestamp,
        receiver,
        sender,
        nonce,
        user_agent,
        START_HEIGHT,
    )
}

use std::net::SocketAddr;

use bitcoin::network::constants::{self, ServiceFlags};
use bitcoin::network::message::{NetworkMessage, RawNetworkMessage};
use bitcoin::network::message_network::VersionMessage;
use bitcoin::network::Address;
use rand::Rng;
use tracing::instrument;

use super::actor::ConnectionActor;
use crate::error::Error;

#[derive(Debug)]
pub struct ConnectionHandle {
    to_actor_sender: tokio::sync::mpsc::Sender<ToConnectionHandle>,
    from_actor_receiver: tokio::sync::broadcast::Receiver<FromConnectionHandle>,
    network: constants::Network,
    peer_address: SocketAddr,
    sender_address: SocketAddr,
    #[allow(dead_code)]
    actor_handle: tokio::task::JoinHandle<()>,
}

impl Drop for ConnectionHandle {
    fn drop(&mut self) {
        tracing::info!("Dropping connection handle");
        self.actor_handle.abort();
    }
}

/// Represents commands and messages that are sent to the actor.
/// Currently, we are only interested in sending messages to the bitcoin node,
/// but this allows for extensible design, the actor could have internal state that reacts on other commands
/// like `Stop` or `Start`, `Restart`. The whole `InitHandshake` could be moved to the actor as well and could be invoked externally.
#[derive(Debug)]
pub enum ToConnectionHandle {
    ToBitcoinNode(RawNetworkMessage),
    InitHandshake { version: VersionMessage },
}

/// Represents messages that are received from the actor.
/// Currently, we are only interested in messages from the bitcoin node,
/// The actor could send other messages like `HandshakeComplete` or `HandshakeFailed`.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum FromConnectionHandle {
    /// Message from the bitcoin node. We are not interested in the magic number,
    /// therefore we are using NetworkMessage instead of RawNetworkMessage.
    FromBitcoinNode(NetworkMessage),
}

impl ConnectionHandle {
    #[instrument(err, fields(peer_address, sender_address, network))]
    pub async fn new(
        peer_address: SocketAddr,
        sender_address: SocketAddr,
        network: constants::Network,
    ) -> Result<Self, Error> {
        tracing::info!("Creating connection to node");
        // The size of mpsc channels before they start blocking
        const CHANNEL_SIZE: usize = 10;

        // Init communication primitives with the actor
        let (to_actor_sender, to_actor_receiver) = tokio::sync::mpsc::channel(CHANNEL_SIZE);
        let (from_actor_sender, from_actor_receiver) =
            tokio::sync::broadcast::channel(CHANNEL_SIZE);

        // Spawn the actor
        let actor =
            ConnectionActor::new(peer_address, to_actor_receiver, from_actor_sender, network)?;
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
    #[instrument(skip(self), err)]
    async fn send_to_node(&self, message: NetworkMessage) -> Result<(), Error> {
        tracing::debug!("Sending message to node: {:?}", message);
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

    #[instrument(level = "debug", skip(self), ret, err)]
    pub async fn receive(&mut self) -> Result<FromConnectionHandle, Error> {
        self.from_actor_receiver
            .recv()
            .await
            .map_err(|_| Error::ActorUnavailable)
    }

    pub async fn send_get_addr(&self) -> Result<(), Error> {
        let message = NetworkMessage::GetAddr;

        self.send_to_node(message).await
    }

    pub async fn init_handshake(&mut self) -> Result<(), Error> {
        let version = build_version_message(&self.peer_address, &self.sender_address);
        let message = ToConnectionHandle::InitHandshake { version };

        self.to_actor_sender
            .send(message)
            .await
            .map_err(|_| Error::ActorSendError)?;

        self.receive_version().await?;
        self.receive_verack().await?;

        Ok(())
    }

    pub async fn receive_version(&mut self) -> Result<VersionMessage, Error> {
        let message = self.receive().await?;
        let FromConnectionHandle::FromBitcoinNode(NetworkMessage::Version(message)) = message else {
            return Err(Error::UnexpectedConnectionMessage(Box::new(message)))
        };

        Ok(message)
    }

    pub async fn receive_verack(&mut self) -> Result<(), Error> {
        let message = self.receive().await?;
        let FromConnectionHandle::FromBitcoinNode(NetworkMessage::Verack) = message else {
            return Err(Error::UnexpectedConnectionMessage(Box::new(message)))
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

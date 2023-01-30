use std::net::SocketAddr;

use models::{BitcoinMessage, Version, Verack};
use tokio::io::AsyncReadExt;
use tokio::net::tcp::WriteHalf;
use tokio::net::TcpStream;
use tracing::instrument;

use crate::error::{self, Error};

use super::actor::ConnectionActor;

#[derive(Debug)]
pub struct ConnectionHandle {
    to_actor_sender: tokio::sync::mpsc::Sender<ToConnectionHandle>,
    from_actor_receiver: tokio::sync::mpsc::Receiver<FromConnectionHandle>,
    #[allow(dead_code)]
    actor_handle: tokio::task::JoinHandle<()>,
}

#[derive(Debug)]
pub enum ToConnectionHandle {
    ToBitcoinNode(BitcoinMessage),
}

#[derive(Debug)]
pub enum FromConnectionHandle {
    FromBitcoinNode(BitcoinMessage),
}


impl ConnectionHandle {
    #[instrument(err, ret)]
    pub async fn new(addr: SocketAddr) -> Result<Self, Error> {
        // Init communication primitives with the actor
        let (to_actor_sender, to_actor_receiver) = tokio::sync::mpsc::channel(10);
        let (from_actor_sender, from_actor_receiver) = tokio::sync::mpsc::channel(10);

        // Init the tcp stream
        let stream = TcpStream::connect(addr).await?;

        // Spawn the actor
        let mut actor = ConnectionActor::new(stream, to_actor_receiver, from_actor_sender);
        let actor_handle = tokio::spawn(async move {
            actor.run().await;
        });
        Ok(Self {
            actor_handle,
            to_actor_sender,
            from_actor_receiver,
        })
    }

    /// Common send method for all messages, will properly map the error types
    #[instrument(skip(self), err, ret)]
    async fn send(&self, msg: ToConnectionHandle) -> Result<(), Error> {
        self.to_actor_sender
            .send(msg)
            .await
            .map_err(|_| Error::ActorSendError)?;
        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    async fn receive(&mut self) -> Result<FromConnectionHandle, Error> {
        self.from_actor_receiver
            .recv()
            .await
            .ok_or(Error::ConnectionDied)
    }

    pub async fn send_version(&self) -> Result<(), Error> {
        self.send(ToConnectionHandle::ToBitcoinNode(BitcoinMessage::Version(
            Version,
        )))
        .await
    }

    pub async fn receive_version(&mut self) -> Result<Version, Error> {
        let msg = self.receive().await?;
        let FromConnectionHandle::FromBitcoinNode(BitcoinMessage::Version(msg)) = msg else {
            return Err(Error::UnexpectedConnectionMessage(msg))
        };

        Ok(msg)
    }

    pub async fn send_verack(&self) -> Result<(), Error> {
        self.send(ToConnectionHandle::ToBitcoinNode(BitcoinMessage::Verack(
            Verack,
        )))
        .await
    }

    pub async fn receive_verack(&mut self) -> Result<Verack, Error> {
        let msg = self.receive().await?;
        let FromConnectionHandle::FromBitcoinNode(BitcoinMessage::Verack(msg)) = msg else {
            return Err(Error::UnexpectedConnectionMessage(msg))
        };

        Ok(msg)
    }
}

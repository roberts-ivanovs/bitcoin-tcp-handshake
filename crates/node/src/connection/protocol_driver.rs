use std::io::Write;
use std::net::{Shutdown, TcpStream};

use bitcoin::consensus::encode;
use bitcoin::network::constants;
use bitcoin::network::message::{NetworkMessage, RawNetworkMessage};
use futures::{pin_mut, Stream, StreamExt};

use super::handle::ToConnectionHandle;
use crate::error::Error;
use crate::FromConnectionHandle;

pub(crate) struct ProtocolDriver {
    pub(crate) write_stream: TcpStream,
    pub(crate) network: constants::Network,
}

impl ProtocolDriver {
    pub(crate) fn new(write_stream: TcpStream, network: constants::Network) -> Self {
        Self {
            write_stream,
            network,
        }
    }

    pub(crate) async fn drive(
        &mut self,
        incoming_commands: impl Stream<Item = ToConnectionHandle>,
        from_node: impl Stream<Item = FromConnectionHandle>,
    ) {
        pin_mut!(incoming_commands);
        pin_mut!(from_node);
        loop {
            let success = tokio::select! {
                msg = incoming_commands.next() => self.handle_incoming_commands(msg),
                msg = from_node.next() => self.handle_messages_from_node(msg),
                else => {
                    tracing::warn!("Both streams are closed");
                    break;
                }
            };
            if success.is_err() {
                break;
            }
        }
    }

    pub(crate) fn handle_messages_from_node(
        &mut self,
        msg: Option<FromConnectionHandle>,
    ) -> Result<(), Error> {
        match msg {
            Some(FromConnectionHandle::FromBitcoinNode(msg)) => {
                match msg {
                    NetworkMessage::Version(_version) => {
                        // NOTE: we can add extra version validation here
                        self.send_blocking(self.verack_message())
                    }
                    // NOTE: add handling for other messages here
                    _ => {
                        // not acting on other messages for now
                        Ok(())
                    }
                }
            }
            None => Err(Error::ActorUnavailable),
        }
    }

    pub(crate) fn handle_incoming_commands(
        &mut self,
        msg: Option<ToConnectionHandle>,
    ) -> Result<(), Error> {
        match msg {
            Some(msg) => {
                match msg {
                    ToConnectionHandle::ToBitcoinNode(msg) => self.send_blocking(msg),
                    ToConnectionHandle::InitHandshake { version } => {
                        // Send version
                        let message = self.version_message(version);
                        self.send_blocking(message)
                    }
                }
            }
            None => Err(Error::ActorUnavailable),
        }
    }

    pub(crate) fn send_blocking(&mut self, msg: RawNetworkMessage) -> Result<(), Error> {
        tokio::task::block_in_place(|| {
            let msg = encode::serialize(&msg);
            self.write_stream.write_all(msg.as_slice())
        })
        .map_err(|_| Error::ActorSendError)
    }

    pub(crate) fn version_message(
        &self,
        version: bitcoin::network::message_network::VersionMessage,
    ) -> RawNetworkMessage {
        RawNetworkMessage {
            magic: self.network.magic(),
            payload: NetworkMessage::Version(version),
        }
    }

    pub(crate) fn verack_message(&self) -> RawNetworkMessage {
        RawNetworkMessage {
            magic: self.network.magic(),
            payload: NetworkMessage::Verack,
        }
    }
}

pub(crate) fn mpsc_to_stream<T>(
    mut receiver: tokio::sync::mpsc::Receiver<T>,
) -> impl Stream<Item = T> {
    async_stream::stream! {
        while let Some(msg) = receiver.recv().await {
            yield msg;
        }
    }
}

pub(crate) fn broadcast_to_stream<T: Clone>(
    mut receiver: tokio::sync::broadcast::Receiver<T>,
) -> impl Stream<Item = T> {
    async_stream::stream! {
        while let Ok(msg) = receiver.recv().await {
            yield msg;
        }
    }
}

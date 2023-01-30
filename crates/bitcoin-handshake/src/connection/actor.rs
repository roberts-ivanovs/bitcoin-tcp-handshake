use std::net::SocketAddr;

use models::{BitcoinMessage, Version, Verack};
use tokio::io::AsyncReadExt;
use tokio::net::tcp::WriteHalf;
use tokio::net::TcpStream;

use crate::error::{self, Error};

use super::{handle::ToConnectionHandle, FromConnectionHandle};

pub struct ConnectionActor {
    stream: TcpStream,
    incoming_messages: tokio::sync::mpsc::Receiver<ToConnectionHandle>,
    outgoing_messages: tokio::sync::mpsc::Sender<FromConnectionHandle>,
}

impl ConnectionActor {
    pub(super) fn new(
        stream: TcpStream,
        incoming_messages: tokio::sync::mpsc::Receiver<ToConnectionHandle>,
        outgoing_messages: tokio::sync::mpsc::Sender<FromConnectionHandle>,
    ) -> Self {
        Self {
            stream,
            incoming_messages,
            outgoing_messages,
        }
    }

    pub(super) async fn run(&mut self) {
        let (mut reader, mut writer) = self.stream.split();

        let mut read_buf = Vec::new();

        // TODO we can move reading and writing to separate tasks
        tokio::select! {
            Ok(res) = reader.read_buf(&mut read_buf) => {
                // TODO: deserialize the message and pretty-print it
                tracing::info!("Read {} bytes", read_buf.len());
                // let _ = self.outgoing_messages.send(FromConnectionHandle::FromBitcoinNode(())).await;
            }
            Some(msg) = self.incoming_messages.recv() => {
                match msg {
                    ToConnectionHandle::ToBitcoinNode(msg) => {

                    }
                }
                // self.handle_incoming_message(msg, &mut writer).await;
                // tracing::info!("Received message from the client");
                // let msg = bitcoin::network::encodable::ConsensusEncodable::consensus_encode(&msg);
                // writer.write_all(&msg).await.unwrap();
                // writer.flush().await.unwrap();
            }
        };

        tracing::info!("Processing over");
    }
}

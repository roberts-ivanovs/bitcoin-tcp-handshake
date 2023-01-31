///! This module contains the actor that handles the connection to the bitcoin node.
///! Sadly [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin) only supports a blocking interface for parsing network messages.
///! There are [open discussions regarding this issue](https://github.com/rust-bitcoin/rust-bitcoin/issues/1251), but for now we have to use a blocking interface.
use std::{io::{BufReader, Write}, net::Shutdown};
use std::net::TcpStream;

use bitcoin::consensus::{encode, Decodable};
use bitcoin::network::message::RawNetworkMessage;

use super::handle::ToConnectionHandle;
use super::FromConnectionHandle;
// use tokio::net::TcpStream;
use crate::error::Error;

pub struct ConnectionActor {
    stream: std::net::TcpStream,
    incoming_messages: tokio::sync::mpsc::Receiver<ToConnectionHandle>,
    outgoing_messages: tokio::sync::mpsc::Sender<FromConnectionHandle>,
}

impl ConnectionActor {
    pub(super) fn new(
        address: std::net::SocketAddr,
        incoming_messages: tokio::sync::mpsc::Receiver<ToConnectionHandle>,
        outgoing_messages: tokio::sync::mpsc::Sender<FromConnectionHandle>,
    ) -> Result<Self, Error> {
        let stream = std::net::TcpStream::connect(address)?;
        Ok(Self {
            stream,
            incoming_messages,
            outgoing_messages,
        })
    }

    /// Start the actor that handles the connection to the bitcoin node. Process messages and build a common state.
    /// NOTE: The common state is not actually built right now, room for improvement.
    pub(super) async fn run(self) {
        let read_stream = self.stream.try_clone().expect("Failed to clone stream");
        let write_stream = self.stream;

        let mut read_task = Self::start_read_task(self.outgoing_messages, read_stream);
        let mut write_task = Self::create_write_task(self.incoming_messages, write_stream);

        loop {
            tokio::select! {
                _ = (&mut read_task) => {
                    tracing::warn!("Read task closed");
                    break;
                }
                _ = (&mut write_task) => {
                    tracing::warn!("Read task closed");
                    break;
                }
                else => {
                    tracing::warn!("Both streams are closed");
                    break;
                }
            };
        }

        read_task.abort();
        write_task.abort();
        tracing::info!("Socket stream processing over");
    }

    /// Read messages from the stream and propagate them to the outside world
    fn start_read_task(
        outgoing_messages: tokio::sync::mpsc::Sender<FromConnectionHandle>,
        read_stream: TcpStream,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut read_stream = BufReader::new(read_stream);
            loop {
                match tokio::task::block_in_place(|| {
                    RawNetworkMessage::consensus_decode(&mut read_stream)
                }) {
                    Ok(msg) => {
                        let sent = outgoing_messages
                            .send(FromConnectionHandle::FromBitcoinNode(msg.payload))
                            .await;
                        if sent.is_err() {
                            tracing::warn!("Outgoing message channel is closed");
                            break;
                        }
                    }
                    Err(err) => {
                        // https://www.reddit.com/r/rust/comments/b095ag/failed_to_fill_whole_buffer_error_with_bufreader/
                        // https://stackoverflow.com/questions/70739158/failed-to-fill-whole-buffer-error-message-when-trying-to-deserialise-an-object
                        tracing::error!("Failed to read-decode message from the stream: {:?}. This usually happens when the node sends some data that we cannot deserialize.", err);
                        let _ = read_stream.get_ref().shutdown(Shutdown::Both);
                        break;
                    }
                }
            }
        })
    }

    /// Receive messages from the outside world and write them to the stream
    fn create_write_task(
        mut incoming_messages: tokio::sync::mpsc::Receiver<ToConnectionHandle>,
        mut write_stream: TcpStream,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(msg) = incoming_messages.recv().await {
                match msg {
                    ToConnectionHandle::ToBitcoinNode(msg) => {
                        let success = tokio::task::block_in_place(|| {
                            let msg = encode::serialize(&msg);
                            write_stream.write_all(msg.as_slice())
                        });

                        if success.is_err() {
                            tracing::warn!("Failed to write message to the stream");
                            let _ = write_stream.shutdown(Shutdown::Both);
                            break;
                        }
                    }
                }
            }
        })
    }
}

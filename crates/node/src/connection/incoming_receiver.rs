use std::io::BufReader;
use std::net::{Shutdown, TcpStream};

use bitcoin::consensus::{self, Decodable};
use bitcoin::network::message::RawNetworkMessage;

use crate::FromConnectionHandle;

pub(crate) struct IncomingReceiver {
    from_node: tokio::sync::broadcast::Sender<FromConnectionHandle>,
    read_stream: BufReader<TcpStream>,
}

impl IncomingReceiver {
    pub(crate) fn new(
        from_node: tokio::sync::broadcast::Sender<FromConnectionHandle>,
        read_stream: TcpStream,
    ) -> Self {
        Self {
            from_node,
            read_stream: BufReader::new(read_stream),
        }
    }

    /// Read messages from the stream and propagate them to the outside world
    pub(crate) async fn broadcast_from_node(&mut self) {
        loop {
            match self.blocking_read() {
                Ok(msg) => {
                    let sent = self
                        .from_node
                        .send(FromConnectionHandle::FromBitcoinNode(msg.payload));
                    if sent.is_err() {
                        tracing::warn!("Outgoing message channel is closed");
                        break;
                    }
                }
                Err(err) => {
                    // https://www.reddit.com/r/rust/comments/b095ag/failed_to_fill_whole_buffer_error_with_bufreader/
                    // https://stackoverflow.com/questions/70739158/failed-to-fill-whole-buffer-error-message-when-trying-to-deserialise-an-object
                    tracing::error!(
                            "Failed to read-decode message from the stream: {:?}. This usually happens when the node sends some data that we cannot deserialize.",
                            err
                        );
                    break;
                }
            }
        }
    }

    fn blocking_read(&mut self) -> Result<RawNetworkMessage, consensus::encode::Error> {
        tokio::task::block_in_place(|| RawNetworkMessage::consensus_decode(&mut self.read_stream))
    }
}

use std::net::Shutdown;

use bitcoin::network::constants;

use super::handle::ToConnectionHandle;
use super::incoming_receiver::IncomingReceiver;
use super::protocol_driver::ProtocolDriver;
use super::{protocol_driver, FromConnectionHandle};
use crate::error::Error;

pub struct ConnectionActor {
    stream: std::net::TcpStream,
    incoming_commands: tokio::sync::mpsc::Receiver<ToConnectionHandle>,
    from_node: tokio::sync::broadcast::Sender<FromConnectionHandle>,
    network: constants::Network,
}

impl ConnectionActor {
    pub(super) fn new(
        address: std::net::SocketAddr,
        incoming_commands: tokio::sync::mpsc::Receiver<ToConnectionHandle>,
        from_node: tokio::sync::broadcast::Sender<FromConnectionHandle>,
        network: constants::Network,
    ) -> Result<Self, Error> {
        let stream = std::net::TcpStream::connect(address)?;
        Ok(Self {
            stream,
            incoming_commands,
            from_node,
            network,
        })
    }

    /// Start the actor that handles the connection to the bitcoin node. Process messages and
    /// build a common state. NOTE: The common state is not actually built right now, room for
    /// improvement.
    pub(super) async fn run(self) {
        let read_stream = self.stream.try_clone().expect("Failed to clone stream");
        let write_stream = self.stream.try_clone().expect("Failed to clone stream");

        let mut protocol_driver_task = Self::init_protocol_driver(
            ProtocolDriver::new(write_stream, self.network),
            self.incoming_commands,
            self.from_node.subscribe(),
        );
        let mut broadcast_task =
            Self::init_node_message_broadcast(IncomingReceiver::new(self.from_node, read_stream));
        loop {
            tokio::select! {
                _ =(&mut broadcast_task) => {
                    tracing::warn!("Read task closed");
                    break;
                }
                _ =(&mut protocol_driver_task) => {
                    tracing::warn!("Read task closed");
                    break;
                }
                else => {
                    tracing::warn!("Both streams are closed");
                    break;
                }
            };
        }

        // Abort the tasks
        broadcast_task.abort();
        protocol_driver_task.abort();

        tracing::warn!("Failed to process stream messages, closing connection");
        let _ = self.stream.shutdown(Shutdown::Both);

        tracing::info!("Socket stream processing over");
    }

    /// Receive messages from the outside world and write them to the stream
    fn init_protocol_driver(
        mut protocol_driver: ProtocolDriver,
        incoming_commands: tokio::sync::mpsc::Receiver<ToConnectionHandle>,
        from_node: tokio::sync::broadcast::Receiver<FromConnectionHandle>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            protocol_driver
                .drive(
                    protocol_driver::mpsc_to_stream(incoming_commands),
                    protocol_driver::broadcast_to_stream(from_node),
                )
                .await;
        })
    }

    /// Receive messages from the outside world and write them to the stream
    fn init_node_message_broadcast(mut receiver: IncomingReceiver) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            receiver.broadcast_from_node().await;
        })
    }
}

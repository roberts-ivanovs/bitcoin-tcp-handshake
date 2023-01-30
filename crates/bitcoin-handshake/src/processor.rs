use tokio::{io::AsyncReadExt, net::tcp::WriteHalf};

use crate::handshake::{ConnectionHandle, Open};


pub enum ToConnectionHandle {
    ToBitcoinNode(()),
}

pub enum FromConnectionHandle {
    FromBitcoinNode(()),
}

impl ConnectionHandle<Open> {
    pub async fn process(&mut self,
        mut incoming_messages: tokio::sync::mpsc::Receiver<ToConnectionHandle>,
        mut outgoing_messages: tokio::sync::mpsc::Sender<FromConnectionHandle>
    ) {
        let (mut reader, mut writer ) = self.split();

        let mut read_buf = Vec::new();

        tokio::select! {
            Ok(res) = reader.read_buf(&mut read_buf) => {
                // TODO: deserialize the message and pretty-print it
                tracing::info!("Read {} bytes", read_buf.len());
                let _ = outgoing_messages.send(FromConnectionHandle::FromBitcoinNode(())).await;
            }
            Some(msg) = incoming_messages.recv() => {
                match msg {
                    ToConnectionHandle::ToBitcoinNode(msg) => {

                        tracing::info!("Received message from the client: {:?}", msg);
                        self.write_and_flush(&read_buf).await;
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


pub struct ConnectionActorHandle {}

struct ConnectionActor {}

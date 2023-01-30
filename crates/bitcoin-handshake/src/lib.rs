mod handshake;
mod processor;
mod types;

use std::net::SocketAddr;

use duplex_tcp_stream::DuplexTcpStream;
use handshake::ConnectionHandle;
use settings::Settings;
use tokio::io::{BufReader, ReadHalf};
use tokio::net::TcpStream;
use tracing::instrument;

pub struct BitcoinNode {
    pub settings: Settings,
}

impl BitcoinNode {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    /// Start the handshake with a foreign node.  The following steps have been taken from [the docs](https://developer.bitcoin.org/devguide/p2p_network.html#connecting-to-peers):
    /// 1. Init a TCP connection to the target node
    /// 2. Start a state machine over the TCP connection
    /// 3. Connecting to a peer is done by sending a "version" message, which contains your version number, block, and current time to the remote node
    /// 4. The remote node responds with its own "version" message.
    /// 5. Then both nodes send a "verack" message to the other node to indicate the connection has been established.
    /// 6. Once connected, the client can send to the remote node getaddr and "addr" messages to gather additional peers.
    pub async fn start(&self) {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        ConnectionHandle::new(self.settings.peer_address())
            .await
            .process_version()
            .await
            .process_verack()
            .await
            .process(receiver);

        todo!()
    }
}

///! Start the handshake with a foreign node.  The following steps have been taken from [the docs](https://developer.bitcoin.org/devguide/p2p_network.html#connecting-to-peers):
///! 1. Init a TCP connection to the target node
///! 2. Start a state machine over the TCP connection
///! 3. Connecting to a peer is done by sending a "version" message, which contains your version number, block, and current time to the remote node
///! 4. The remote node responds with its own "version" message.
///! 5. Then both nodes send a "verack" message to the other node to indicate the connection has been established.
///! 6. Once connected, the client can send to the remote node getaddr and "addr" messages to gather additional peers.
mod error;
mod connection;

use std::marker::PhantomData;

use connection::{ConnectionHandle, FromConnectionHandle};
use error::Error;
use settings::Settings;
use tracing::instrument;

pub struct BitcoinConnector {
    settings: Settings,
}

impl BitcoinConnector {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    #[instrument(skip(self), err, ret)]
    pub async fn connect(self) -> Result<BitcoinConnection<PreHandshake>, Error> {
        let peer_address = self.settings.peer_address();
        let sender_address = self.settings.sender_address();
        let peer_network = self.settings.peer_network();

        let connection_handle =
            ConnectionHandle::new(peer_address, sender_address, peer_network).await?;
        let connection = BitcoinConnection::<PreHandshake>::new(self.settings, connection_handle);

        Ok(connection)
    }
}

#[derive(Debug)]
pub struct Connected;

#[derive(Debug)]
pub struct PreHandshake;

#[derive(Debug)]
pub struct BitcoinConnection<T> {
    settings: Settings,
    connection: ConnectionHandle,
    _type: PhantomData<T>,
}

impl BitcoinConnection<PreHandshake> {
    pub(crate) fn new(settings: Settings, connection: ConnectionHandle) -> Self {
        Self {
            settings,
            connection,
            _type: PhantomData,
        }
    }

    #[instrument(skip(self), err, ret)]
    pub async fn perform_handshake(mut self) -> Result<BitcoinConnection<Connected>, Error> {
        // TODO introduce timeout for handshake tokio::select! with tokio::sleep
        self.connection.send_version().await?;
        self.connection.receive_version().await?;
        self.connection.send_verack().await?;
        self.connection.receive_verack().await?;

        let connection = BitcoinConnection::<Connected>::new(self.settings, self.connection);
        Ok(connection)
    }
}

impl BitcoinConnection<Connected> {
    pub(self) fn new(settings: Settings, connection: ConnectionHandle) -> Self {
        Self {
            settings,
            connection,
            _type: PhantomData,
        }
    }

    pub async fn send_get_addr(&self) -> Result<(), Error> {
        self.connection.send_get_addr().await
    }

    pub async fn receive(&mut self) -> Result<FromConnectionHandle, Error> {
        self.connection.receive().await
    }
}

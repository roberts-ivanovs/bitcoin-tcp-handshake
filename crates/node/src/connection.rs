mod actor;
mod handle;
mod incoming_receiver;
mod protocol_driver;

use std::marker::PhantomData;

use error::Error;
pub use handle::{ConnectionHandle, FromConnectionHandle};
use settings::Settings;
use tracing::instrument;

use crate::error;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct BitcoinConnector {
    settings: Settings,
}

impl BitcoinConnector {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    /// Start a new connection to the bitcoin node.
    #[instrument(skip(self), err)]
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

/// Typestate pattern enforced connection. To ensure that the handshake is performed before sending any subsequent messages
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

    /// Initiate the handshake with the peer. This will send the version message and wait for the verack message.
    #[instrument(skip(self), err)]
    pub async fn perform_handshake(mut self) -> Result<BitcoinConnection<Connected>, Error> {
        // handshake data specific validation should happen at this level.
        tracing::info!("Initiating handshake");
        self.connection.init_handshake().await?;

        tracing::info!("Handshake completed successfully");
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

    /// Expose receive method from the connection handle for demo purposes. In a real application this would be replaced by a more abstract interface.
    pub async fn receive(&mut self) -> Result<FromConnectionHandle, Error> {
        self.connection.receive().await
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Connected;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct PreHandshake;

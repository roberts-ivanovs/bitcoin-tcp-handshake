use std::net::SocketAddr;

use config::{Config, Environment, File, FileFormat};
use serde::Deserialize;

use crate::configs::BASE_CONFIG;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    peer_address: SocketAddr,
    sender_address: SocketAddr,
    peer_network: bitcoin::Network,
}

impl Settings {
    pub fn new() -> Self {
        Config::builder()
            .add_source(File::from_str(BASE_CONFIG, FileFormat::Toml))
            // Overrides from the environment
            .add_source(Environment::default())
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }

    pub fn peer_address(&self) -> SocketAddr {
        self.peer_address
    }

    pub fn peer_network(&self) -> bitcoin::Network {
        self.peer_network
    }

    pub fn sender_address(&self) -> SocketAddr {
        self.sender_address
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

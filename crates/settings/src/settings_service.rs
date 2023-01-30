use std::net::SocketAddr;

use config::{Config, Environment, File, FileFormat};
use serde::{ Deserialize};

use crate::configs::BASE_CONFIG;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    peer_address: SocketAddr,
    response_timeout_ms: u64,
    connection_timeout_ms: u64,
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
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

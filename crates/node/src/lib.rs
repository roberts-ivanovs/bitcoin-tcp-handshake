///! A simple library that allows to instantiate a new connection to a bitcoin node.
mod connection;
mod error;

pub use bitcoin::network;
pub use connection::{BitcoinConnector, FromConnectionHandle};

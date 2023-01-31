mod trace;

use std::process;

use anyhow::Result;
use node::network::message::NetworkMessage;
use node::{BitcoinConnector, FromConnectionHandle};

#[tokio::main]
async fn main() -> Result<()> {
    // Init settings
    let settings = settings::Settings::new();

    // Init tracing
    trace::init_tracing();
    tracing::info!("(set RUST_LOG=debug mode to view all transmitting messages)");

    // Connect to the node
    let mut node = BitcoinConnector::new(settings)
        .connect()
        .await?
        .perform_handshake()
        .await?;

    // Query extra data from the node
    tracing::info!("Query extra data from the node to demonstrate that the connection is active");
    node.send_get_addr().await?;

    // Receive messages from the node. Usually I wouldn't expose the receiver to the end-user, but this is just for demo purposes.
    while let Ok(msg) = node.receive().await {
        // Break the loop when we receive the Addr message
        if let FromConnectionHandle::FromBitcoinNode(NetworkMessage::Addr(msg)) = msg {
            tracing::info!("The Addr message has been received, len - {}", msg.len());
            break;
        }
    }

    process::exit(0);
}

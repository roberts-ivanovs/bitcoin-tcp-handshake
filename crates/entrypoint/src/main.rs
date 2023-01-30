mod trace;

use bitcoin_handshake::BitcoinConnector;
use tracing::info;
use tracing::metadata::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Init settings
    let settings = settings::Settings::new();

    // Init tracing
    trace::init_tracing();

    // Connect to the node
    let mut node = BitcoinConnector::new(settings)
        .connect()
        .await?
        .process_handshake()
        .await?;

    // Query extra data from the node
    node.send_get_addr().await?;
    tracing::info!("The response usually takes a few seconds to arrive...");

    // Receive messages from the node
    while let Ok(msg) = node.receive().await {
        tracing::info!(msg = ?msg);
    }

    Ok(())
}

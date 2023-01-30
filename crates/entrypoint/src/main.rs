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
    let node = BitcoinConnector::new(settings)
        .connect()
        .await?
        .process_handshake()
        .await?;

    // Query extra data from the node
    let addr_info = node.get_addr().await;
    tracing::info!(
        r#"
    The following message acts as proof that a successful handshake has been
    established and extra data can be queried: {:?}"#,
        addr_info
    );

    Ok(())
}

mod trace;

use anyhow::Result;
use bitcoin_handshake::{network::message::NetworkMessage, BitcoinConnector, FromConnectionHandle};

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
        .perform_handshake()
        .await?;

    // Query extra data from the node
    node.send_get_addr().await?;
    tracing::info!("The response usually takes a few seconds to arrive...");

    // Receive messages from the node
    while let Ok(msg) = node.receive().await {
        tracing::info!(msg = ?msg);

        // Stop the loop when we receive the Addr message
        if matches!(
            msg,
            FromConnectionHandle::FromBitcoinNode(NetworkMessage::Addr(_))
        ) {
            break;
        }
    }

    // TODO the node is not closed properly. process hangs up
    Ok(())
}

use bitcoin_handshake::BitcoinNode;
use tracing::{info, metadata::LevelFilter};
use tracing_subscriber::{prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    // Init settings
    let settings = settings::Settings::new();

    // Init tracing
    init_tracing();

    let node = BitcoinNode::new(settings);
    node.start().await
}


/// Construct a subscriber that prints JSON formatted traces to stdout
fn init_tracing() {
    let env=  EnvFilter::builder()
         .with_default_directive(LevelFilter::INFO.into())
         .with_env_var("RUST_LOG")
         .from_env_lossy();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(false)
        .json();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env)
        .init();
}

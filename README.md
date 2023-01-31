# Bitcoin node handshake

## Pre-requisites

[Install rust](https://www.rust-lang.org/tools/install)

## Running the handshake tool
TLDR
```bash
PEER_ADDRESS="$(dig seed.bitcoin.sipa.be +short | head -n 1):8333" && cargo run -p node --example handshake
# OR
PEER_ADDRESS="2.59.236.56:8333" cargo run -p node --example handshake
# To view debug logs (will log received messages from node)
RUST_LOG="debug" PEER_ADDRESS="2.59.236.56:8333" cargo run -p node --example handshake
```

### Longer version
This is a simple tool to test the handshake of a Bitcoin node. This example code *will not* perform a network scan to discover nodes. You will need to know the IP address of the node you want to test.
To get the IP address of a node, we can manually perform the steps as described in the [Bitcoin developer guide](https://developer.bitcoin.org/devguide/p2p_network.html):

```bash
# Get a list of IP addresses from a DNS seed
$ dig seed.bitcoin.sipa.be +short
```

Then, run the handshake tool, by setting one of the IP addresses to the `PEER_ADDRESS` environment variable:

```bash
# Option 1
PEER_ADDRESS="2.59.236.56:8333" cargo run -p node --example handshake
# Option 2 - alter the config.base.toml. This will be the default address, but requires a recompilation
cargo run -p node --example handshake
```

## Architecture

### Handshake
Start the handshake with a foreign node. The following steps have been taken from [the docs](https://developer.bitcoin.org/devguide/p2p_network.html#connecting-to-peers):
1. Init a TCP connection to the target node
2. Connecting to a peer is done by sending a "version" message, which contains your version number, block, and current time to the remote node
3. The remote node responds with its own "version" message.
4. Then both nodes send a "verack" message to the other node to indicate the connection has been established.
5. Once connected, the client can send to the remote node getaddr and "addr" messages to gather additional peers.

### Code overview
1. The `settings` crate will parse the env variables, as well as the local `config.base.toml` config file. It provides a `Settings` struct that can be used to configure the application.
2. The `entrypoint` crate is the entrypoint of the application. It is responsible for instantiating a new connection to the remote node, and starting the handshake.
3. The `node` crate is responsible for the handshake. It performs creates a TCP connection, performs the all of the message wiring and data parsing, and performs the handshake.

#### Implementation details
1. Duplex (bidirectional) streams are used to communicate with the remote node.
2. Sadly [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin) only supports a blocking interface for parsing network messages, as it is tied to the `io::Read` trait, which is not implemented by tokio (and thus Tokio implementation of `TcpStream` ). The blocking TCP stream operations are delegated to the blocking thread of Tokio using `tokio::task::block_in_place()`.
3. The stream processor is implemented using the actor pattern with [Tokio (pattern described here)](https://ryhl.io/blog/actors-with-tokio/), this allows for flexibility where we can have stateful logic and asynchronous tasks that work with the TCP connection (e.g. periodic ping-pong messages, automated responses for incoming messages). The rest of the system can consume and build a more sophisticated model top of this message processing actor as needed.

 <!-- TODO add tests -->
 <!-- TODO derive all the common traits for structs -->


## Development
```bash
# Aggressive formatting
cargo +nightly fmt --all
cargo clippy --fix --allow-dirty --allow-staged --workspace --bins --tests
```

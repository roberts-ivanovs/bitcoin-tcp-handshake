# Bitcoin node handshake


## Running the handshake tool

TLDR
```bash
PEER_ADDRESS="$(dig seed.bitcoin.sipa.be +short | head -n 1):8333" && cargo run -p entrypoint
```

---

This is a simple tool to test the handshake of a Bitcoin node. This example code *will not* perform a network scan to discover nodes. You will need to know the IP address of the node you want to test.
To get the IP address of a node, we can manually perform the steps as described in the [Bitcoin developer guide](https://developer.bitcoin.org/devguide/p2p_network.html):

```bash
# Get a list of IP addresses from a DNS seed
$ dig seed.bitcoin.sipa.be +short
```

Then, run the handshake tool, by setting one of the IP addresses to the `PEER_ADDRESS` environment variable:

```bash
# Option 1
PEER_ADDRESS="2.59.236.56:8333" cargo run -p entrypoint
# Option 2 - alter the config.base.toml. This will be the default address, but requires a recompilation
cargo run -p entrypoint
```

## Architecture

<!-- TODO -->
<!--
1. abstraction layers
2. duplex streams
3. explain the actor model
4. common crates and approaches
 -->

 <!-- TODO can we drop nightly? -->

 <!-- TODO setup formatting -->

 <!-- TODO add docs -->
 <!-- TODO add tests -->
 <!-- TODO protect node under interior mutability -->


## Development


```bash
# Aggressive formatting
cargo +nightly fmt --all
cargo clippy --fix --allow-dirty --allow-staged --workspace --bins --tests
```

#!/usr/bin/env bash

set -e

# install rust
curl https://sh.rustup.rs -sSf | sh -s -- -y
source $HOME/.cargo/env
rustup default $RUST_VERSION
rustup update

# for DB migrations
cargo install diesel_cli --no-default-features --features "postgres"

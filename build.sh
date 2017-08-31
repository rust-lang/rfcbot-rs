#!/usr/bin/env bash

set -e

# build binary
cargo build --release

cp target/release/rust-dashboard ./rust-dashboard/usr/bin/rust-dashboard

# build .deb
dpkg-deb --build rust-dashboard

# cleanup
rm ./rust-dashboard/usr/bin/rust-dashboard

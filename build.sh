#!/usr/bin/env bash

set -e

# prep directories
mkdir -p ./rust-dashboard/usr/share/rust-dashboard/www

# build binary
cargo build --release

cp target/release/rust-dashboard ./rust-dashboard/usr/bin/rust-dashboard

# build frontend
cd front
ember build --environment=production
cp -R ./dist/* ../rust-dashboard/usr/share/rust-dashboard/www/
cd ..

# build .deb
dpkg-deb --build rust-dashboard

# cleanup
rm -dr ./rust-dashboard/usr/share/rust-dashboard/www
rm ./rust-dashboard/usr/bin/rust-dashboard

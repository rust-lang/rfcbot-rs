#!/usr/bin/env bash

# default dashboard config
export DATABASE_URL=postgres://localhost/dashboard
export DATABASE_POOL_SIZE=10
export RUST_LOG=debug,hyper=info,rustc=error,cargo=error
export GITHUB_SCRAPE_INTERVAL=10
export RELEASES_SCRAPE_INTERVAL=720
export BUILDBOT_SCRAPE_INTERVAL=80
export SERVER_PORT=8080

# VM config for cargo
export CARGO_TARGET_DIR=/rust-dashboard/target

echo "Environment variables set!"

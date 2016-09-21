#!/usr/bin/env bash

# default dashboard config
export DATABASE_URL=postgres://vagrant:hunter2@localhost/dashboard
export DATABASE_POOL_SIZE=10
export RUST_LOG=debug,hyper=info,rustc=error,cargo=error
export GITHUB_SCRAPE_INTERVAL=2
export RELEASES_SCRAPE_INTERVAL=720
export BUILDBOT_SCRAPE_INTERVAL=80
export SERVER_PORT=8080
export POST_COMMENTS=false

# VM config for cargo
export CARGO_TARGET_DIR=/rust-dashboard/target

export RUST_BACKTRACE=1
export RUST_NEW_ERROR_FORMAT=1

export PATH=$PATH:$HOME/.cargo/bin
export PS1="\[\033[01;37m\]\$? \$(if [[ \$? == 0 ]]; then echo \"\[\033[01;32m\]\342\234\223\"; else echo \"\[\033[01;31m\]\342\234\227\"; fi) $(if [[ ${EUID} == 0 ]]; then echo '\[\033[01;31m\]\h'; else echo '\[\033[01;32m\]\u@\h'; fi)\[\033[01;34m\] \w \$\[\033[00m\] "

echo "Environment variables set!"

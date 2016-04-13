// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

extern crate chrono;
#[macro_use]
extern crate hyper;

mod config;
mod github;

fn main() {
    let cfg = match config::init() {
        Ok(cfg) => cfg,
        Err(missing) => {
            panic!("Unable to load environment variables: {:?}", missing);
        }
    };

    let mut gh = github::client::Client::from(&cfg);
}

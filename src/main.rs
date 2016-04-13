// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

mod config;

fn main() {
    let cfg = match config::init() {
        Ok(cfg) => cfg,
        Err(missing) => {
            panic!("Unable to load environment variables: {:?}", missing);
        }
    };

    println!("{:#?}", cfg);
}

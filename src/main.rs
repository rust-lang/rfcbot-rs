// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(diesel_codegen, serde_macros)]

extern crate chrono;
extern crate clap;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;

mod config;
mod domain;
mod github;

use chrono::{DateTime, TimeZone, UTC};

fn main() {
    fn make_date_time(date_str: &str) -> Result<DateTime<UTC>, chrono::ParseError> {
        UTC.datetime_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S")
    }

    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
                      .version(env!("CARGO_PKG_VERSION"))
                      .author(env!("CARGO_PKG_AUTHORS"))
                      .about(env!("CARGO_PKG_DESCRIPTION"))
                      .subcommand(clap::SubCommand::with_name("bootstrap")
                                      .about("bootstraps the database")
                                      .arg(clap::Arg::with_name("since")
                                               .index(1)
                                               .required(true)
                                               .help("Date in YYYY-MM-DD format.")
                                               .validator(|d| {
                                                   make_date_time(&d)
                                                      .map(|_| ())
                                                      .map_err(|e| format!("Date must be in YYYY-MM-DD format ({:?})", e))
                                               })))
                      .get_matches();

    if let Some(matches) = matches.subcommand_matches("bootstrap") {
        let start = make_date_time(matches.value_of("since").unwrap()).unwrap();

        println!("{:?}", github::ingest_since(start).map(|()| "Ingestion succesful."));
    }
}

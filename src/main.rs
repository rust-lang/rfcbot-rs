// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(diesel_codegen, dotenv_macros, serde_macros)]

extern crate chrono;
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde;
extern crate serde_json;

mod config;
mod domain;
mod github;

use chrono::{DateTime, TimeZone, UTC};
use clap::{App, Arg, ArgMatches, SubCommand};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

use config::CONFIG;

// initialize the database connection pool
lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = {

        let config = r2d2::Config::builder()
                         .pool_size(CONFIG.db_pool_size)
                         .build();

        let manager = ConnectionManager::<PgConnection>::new(CONFIG.db_url.clone());
        Pool::new(config, manager).expect("Failed to create database connection pool.")
    };
}

fn main() {
    dotenv::dotenv().ok();

    let args = init_cli();

    if let Some(args) = args.subcommand_matches("bootstrap") {
        // OK to unwrap, this has already been validated by clap
        let start = make_date_time(args.value_of("since").unwrap()).unwrap();

        println!("{:?}",
                 github::ingest_since(start).map(|()| "Ingestion succesful."));
    } else {
        use domain::schema::githubuser::dsl::*;
        let users: Vec<domain::github::GitHubUser> = githubuser.load(&*DB_POOL.get().unwrap())
                                                               .unwrap();
        println!("{:?}", users);
    }
}

fn make_date_time(date_str: &str) -> Result<DateTime<UTC>, chrono::ParseError> {
    UTC.datetime_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S")
}

fn init_cli<'a>() -> ArgMatches<'a> {
    let bootstrap = SubCommand::with_name("bootstrap")
                        .about("bootstraps the database")
                        .arg(Arg::with_name("since")
                                 .index(1)
                                 .required(true)
                                 .help("Date in YYYY-MM-DD format.")
                                 .validator(|d| {
                                     make_date_time(&d)
                                         .map(|_| ())
                                         .map_err(|e| {
                                             format!("Date must be in YYYY-MM-DD format ({:?})", e)
                                         })
                                 }));

    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(bootstrap)
        .get_matches()
}

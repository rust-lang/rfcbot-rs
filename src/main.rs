// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(diesel_codegen, dotenv_macros, serde_macros)]

extern crate chrono;
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
#[macro_use]
extern crate hyper;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde;
extern crate serde_json;

mod buildbot;
mod config;
mod domain;
mod error;
mod github;
mod releases;

use chrono::{DateTime, Local, TimeZone, UTC};
use clap::{App, Arg, ArgMatches, SubCommand};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use env_logger::LogBuilder;
use log::LogRecord;
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

use config::CONFIG;

// initialize the database connection pool
lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = {
        info!("Initializing database connection pool.");

        let config = r2d2::Config::builder()
                         .pool_size(CONFIG.db_pool_size)
                         .build();

        let manager = ConnectionManager::<PgConnection>::new(CONFIG.db_url.clone());
        match Pool::new(config, manager) {
            Ok(p) => {
                info!("DB connection pool established.");
                p
            },
            Err(why) => {
                error!("Failed to establish DB connection pool: {}", why);
                panic!("Error creating connection pool.");
            }
        }
    };
}

fn main() {
    // init environment variables, CLI, and logging
    dotenv::dotenv().ok();
    let args = init_cli();

    LogBuilder::new()
        .format(|rec: &LogRecord| {
            let loc = rec.location();
            format!("[{} {}:{} {}] {}",
                    rec.level(),
                    loc.module_path(),
                    loc.line(),
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    rec.args())
        })
        .parse(&std::env::var("RUST_LOG").unwrap_or("info".to_string()))
        .init()
        .unwrap();

    debug!("Logging initialized.");
    let _ = CONFIG.check();
    let _ = DB_POOL.get().expect("Unable to test connection pool.");

    if let Some(args) = args.subcommand_matches("bootstrap") {
        // OK to unwrap, this has already been validated by clap
        let start = make_date_time(args.value_of("since").unwrap())
                        .unwrap_or(UTC.ymd(2015, 5, 15).and_hms(0, 0, 0));

        let source = args.value_of("source").unwrap();

        match source {
            "github" => {
                info!("Bootstrapping GitHub data since {}", start);
                info!("{:#?}",
                      github::ingest_since(start).map(|()| "Ingestion succesful."))
            }

            "releases" => {
                info!("Bootstrapping release channel data since {}.", start);
                info!("{:#?}",
                      releases::ingest_releases_since(start).map(|()| "Ingestion successful."));
            }

            "buildbot" => {
                info!("Bootstrapping buildbot data.");
                info!("{:#?}",
                      buildbot::ingest().map(|()| "Ingestion successful."));
            }

            _ => error!("Invalid scraping source specified."),
        }
    } else {
        unimplemented!();
    }
}

fn make_date_time(date_str: &str) -> Result<DateTime<UTC>, chrono::ParseError> {
    UTC.datetime_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S")
}

fn init_cli<'a>() -> ArgMatches<'a> {
    let bootstrap = SubCommand::with_name("bootstrap")
                        .about("bootstraps the database")
                        .arg(Arg::with_name("source")
                                 .index(1)
                                 .required(true)
                                 .help("Data source to scrape ('all' for all)."))
                        .arg(Arg::with_name("since")
                                 .index(2)
                                 .required(true)
                                 .help("Date in YYYY-MM-DD format.")
                                 .validator(|d| {
                                     match &*d {
                                         "all" => Ok(()),
                                         _ => {
                                             make_date_time(&d)
                                                 .map(|_| ())
                                                 .map_err(|e| {
                                                     format!("Date must be in YYYY-MM-DD format \
                                                              ({:?})",
                                                             e)
                                                 })
                                         }
                                     }
                                 }));

    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(bootstrap)
        .get_matches()
}

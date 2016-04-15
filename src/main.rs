// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

#![feature(custom_attribute, custom_derive, plugin)]
#![plugin(diesel_codegen, serde_macros)]

extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate hyper;
extern crate serde;
extern crate serde_json;

mod config;
mod domain;
mod github;

use chrono::{TimeZone, UTC};

fn main() {
    let cfg = match config::init() {
        Ok(cfg) => cfg,
        Err(missing) => {
            panic!("Unable to load environment variables: {:?}", missing);
        }
    };

    let gh = github::client::Client::from(&cfg).unwrap();

    let start_datetime = UTC.ymd(2016, 4, 15).and_hms(0, 0, 0);

    println!("fetching all rust-lang/rust issues and comments since {}", start_datetime);
    let issues = gh.issues_since(start_datetime);
    let comments = gh.comments_since(start_datetime);

    if let (Ok(issues), Ok(comments)) = (issues, comments) {
        println!("num issues updated since {}: {:?}",
                 &start_datetime,
                 issues.len());
        println!("num comments updated since {}: {:?}",
                 &start_datetime,
                 comments.len());
    } else {
        println!("ERROR retrieving issues and comments. You should probably add more error \
                  output.");
    }
}

// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).
//! Configuration environment variables for rfcbot.
//!
//! Note that you can configure the Rocket web server using environment variables like
//! `ROCKET_PORT`, according to the Rocket
//! [configuration guide](https://rocket.rs/guide/configuration/).
//!
//! Here are the variables rfcbot expects to see in its environment:
//!
//! * `DATABASE_URL`: postgres database URL
//! * `DATABASE_POOL_SIZE`: number of connections to maintain in the pool
//! * `GITHUB_ACCESS_TOKEN`: your access token from GitHub. See
//!   [this page](https://help.github.com/articles/creating-an-access-token-for-command-line-use/)
//!   for more information. You shouldn't need to check any of the boxes for granting scopes when
//!   creating it.
//! * `GITHUB_USER_AGENT`: the UA string to send to GitHub (they request that you send your GitHub
//!   username or the app name you registered for the client ID)
//! * `GITHUB_WEBHOOK_SECRETS`: a comma-delimited string of the secrets used for any ingestion
//!   webhooks. The webhook handler will attempt to validate any POST'd webhook against each secret
//!   until it either finds a matching one or runs out.
//! * `RUST_LOG`: the logging configuration for [env_logger](https://crates.io/crates/env_logger).
//!   If you're unfamiliar, you can read about it in the documentation linked on crates.io. If it's
//!   not defined, logging will default to `info!()` and above.
//! * `GITHUB_SCRAPE_INTERVAL`: time (in minutes) to wait in between GitHub scrapes (scraping is
//!   disabled if this environment variable is omitted)
//! * `POST_COMMENTS`: whether to post RFC bot comments on issues -- either `true` or `false`. Be
//!   very careful setting to true when testing -- it will post comments using whatever account is
//!   associated with the GitHub API key you provide.

use std::collections::BTreeMap;
use std::env;

pub const RFC_BOT_MENTION: &str = "@rfcbot";
pub const GH_ORGS: [&str; 3] = ["rust-lang", "rust-lang-nursery", "rust-lang-deprecated"];

lazy_static! {
    pub static ref CONFIG: Config = {
        match init() {
            Ok(c) => {
                info!("Configuration parsed from environment variables.");
                c
            }
            Err(missing) => {
                error!("Unable to load environment variables {:?}", missing);
                panic!("Unable to load environment variables {:?}", missing);
            }
        }
    };
}

#[derive(Debug)]
pub struct Config {
    pub db_url: String,
    pub db_pool_size: u32,
    pub github_access_token: String,
    pub github_user_agent: String,
    pub github_webhook_secrets: Vec<String>,
    pub github_interval_mins: Option<u64>,
    pub post_comments: bool,
}

impl Config {
    pub fn check(&self) -> bool {
        !self.db_url.is_empty()
            && !self.github_access_token.is_empty()
            && !self.github_user_agent.is_empty()
    }
}

const DB_URL: &str = "DATABASE_URL";
const DB_POOL_SIZE: &str = "DATABASE_POOL_SIZE";
const GITHUB_TOKEN: &str = "GITHUB_ACCESS_TOKEN";
const GITHUB_WEBHOOK_SECRETS: &str = "GITHUB_WEBHOOK_SECRETS";
const GITHUB_UA: &str = "GITHUB_USER_AGENT";
const GITHUB_INTERVAL: &str = "GITHUB_SCRAPE_INTERVAL";
const POST_COMMENTS: &str = "POST_COMMENTS";

// this is complex, but we'll shortly need a lot more config items
// so checking them automagically seems like a nice solution
pub fn init() -> Result<Config, Vec<&'static str>> {
    let mut vars: BTreeMap<&'static str, Result<String, _>> = BTreeMap::new();
    [
        DB_URL,
        DB_POOL_SIZE,
        GITHUB_TOKEN,
        GITHUB_WEBHOOK_SECRETS,
        GITHUB_UA,
        POST_COMMENTS,
    ]
    .iter()
    .for_each(|var| {
        vars.insert(var, env::var(var));
    });

    let all_found = vars.iter().all(|(_, v)| v.is_ok());
    if all_found {
        let mut vars = vars
            .into_iter()
            .map(|(k, v)| (k, v.unwrap()))
            .collect::<BTreeMap<_, _>>();

        let db_url = vars.remove(DB_URL).unwrap();
        let db_pool_size = vars.remove(DB_POOL_SIZE).unwrap().parse::<u32>();
        let db_pool_size = ok_or!(db_pool_size, throw!(vec![DB_POOL_SIZE]));

        let gh_token = vars.remove(GITHUB_TOKEN).unwrap();
        let gh_ua = vars.remove(GITHUB_UA).unwrap();

        let gh_interval = if let Ok(val) = env::var(GITHUB_INTERVAL) {
            Some(ok_or!(val.parse::<u64>(), throw!(vec![GITHUB_INTERVAL])))
        } else {
            None
        };

        let post_comments = vars.remove(POST_COMMENTS).unwrap().parse::<bool>();
        let post_comments = ok_or!(post_comments, throw!(vec![POST_COMMENTS]));

        let webhook_secrets = vars.remove(GITHUB_WEBHOOK_SECRETS).unwrap();
        let webhook_secrets = webhook_secrets.split(',').map(String::from).collect();

        Ok(Config {
            db_url,
            db_pool_size,
            github_access_token: gh_token,
            github_user_agent: gh_ua,
            github_webhook_secrets: webhook_secrets,
            github_interval_mins: gh_interval,
            post_comments,
        })
    } else {
        Err(vars
            .iter()
            .filter(|&(_, v)| v.is_err())
            .map(|(&k, _)| k)
            .collect())
    }
}

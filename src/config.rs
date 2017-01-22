// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std::collections::BTreeMap;
use std::env;

pub const RFC_BOT_MENTION: &'static str = "@rfcbot";

lazy_static! {
    pub static ref CONFIG: Config = {
        match init() {
            Ok(c) => {
                info!("Configuration parsed from environment variables.");
                c
            },
            Err(missing) => {
                error!("Unable to load environment variables {:?}", missing);
                panic!("Unable to load environment variables {:?}", missing);
            },
        }
    };
}

#[derive(Debug)]
pub struct Config {
    pub server_port: u32,
    pub db_url: String,
    pub db_pool_size: u32,
    pub github_access_token: String,
    pub github_user_agent: String,
    pub github_webhook_secrets: Vec<String>,
    pub github_interval_mins: u64,
    pub release_interval_mins: u64,
    pub buildbot_interval_mins: u64,
    pub post_comments: bool,
}

impl Config {
    pub fn check(&self) -> bool {
        self.db_url.len() > 0 && self.github_access_token.len() > 0 &&
        self.github_user_agent.len() > 0
    }
}

const SERVER_PORT: &'static str = "SERVER_PORT";
const DB_URL: &'static str = "DATABASE_URL";
const DB_POOL_SIZE: &'static str = "DATABASE_POOL_SIZE";
const GITHUB_TOKEN: &'static str = "GITHUB_ACCESS_TOKEN";
const GITHUB_WEBHOOK_SECRETS: &'static str = "GITHUB_WEBHOOK_SECRETS";
const GITHUB_UA: &'static str = "GITHUB_USER_AGENT";
const GITHUB_INTERVAL: &'static str = "GITHUB_SCRAPE_INTERVAL";
const RELEASES_INTERVAL: &'static str = "RELEASES_SCRAPE_INTERVAL";
const BUILDBOT_INTERVAL: &'static str = "BUILDBOT_SCRAPE_INTERVAL";
const POST_COMMENTS: &'static str = "POST_COMMENTS";

// this is complex, but we'll shortly need a lot more config items
// so checking them automagically seems like a nice solution
pub fn init() -> Result<Config, Vec<&'static str>> {

    let mut vars: BTreeMap<&'static str, Result<String, _>> = BTreeMap::new();
    let keys = vec![SERVER_PORT,
                    DB_URL,
                    DB_POOL_SIZE,
                    GITHUB_TOKEN,
                    GITHUB_WEBHOOK_SECRETS,
                    GITHUB_UA,
                    GITHUB_INTERVAL,
                    RELEASES_INTERVAL,
                    BUILDBOT_INTERVAL,
                    POST_COMMENTS];

    for var in keys.into_iter() {
        vars.insert(var, env::var(var));
    }

    let all_found = vars.iter().all(|(_, v)| v.is_ok());
    if all_found {
        let mut vars = vars.into_iter()
            .map(|(k, v)| (k, v.unwrap()))
            .collect::<BTreeMap<_, _>>();

        let port = vars.remove(SERVER_PORT).unwrap();
        let port = match port.parse::<u32>() {
            Ok(p) => p,
            Err(_) => return Err(vec![SERVER_PORT]),
        };

        let db_url = vars.remove(DB_URL).unwrap();
        let db_pool_size = vars.remove(DB_POOL_SIZE).unwrap();
        let db_pool_size = match db_pool_size.parse::<u32>() {
            Ok(size) => size,
            Err(_) => return Err(vec![DB_POOL_SIZE]),
        };

        let gh_token = vars.remove(GITHUB_TOKEN).unwrap();
        let gh_ua = vars.remove(GITHUB_UA).unwrap();

        let gh_interval = vars.remove(GITHUB_INTERVAL).unwrap();
        let gh_interval = match gh_interval.parse::<u64>() {
            Ok(interval) => interval,
            Err(_) => return Err(vec![GITHUB_INTERVAL]),
        };

        let rel_interval = vars.remove(RELEASES_INTERVAL).unwrap();
        let rel_interval = match rel_interval.parse::<u64>() {
            Ok(interval) => interval,
            Err(_) => return Err(vec![RELEASES_INTERVAL]),
        };

        let bb_interval = vars.remove(BUILDBOT_INTERVAL).unwrap();
        let bb_interval = match bb_interval.parse::<u64>() {
            Ok(interval) => interval,
            Err(_) => return Err(vec![BUILDBOT_INTERVAL]),
        };

        let post_comments = vars.remove(POST_COMMENTS).unwrap();
        let post_comments = match post_comments.parse::<bool>() {
            Ok(pc) => pc,
            Err(_) => return Err(vec![POST_COMMENTS]),
        };

        let webhook_secrets = vars.remove(GITHUB_WEBHOOK_SECRETS).unwrap();
        let webhook_secrets = webhook_secrets.split(',').map(String::from).collect();

        Ok(Config {
            server_port: port,
            db_url: db_url,
            db_pool_size: db_pool_size,
            github_access_token: gh_token,
            github_user_agent: gh_ua,
            github_webhook_secrets: webhook_secrets,
            github_interval_mins: gh_interval,
            release_interval_mins: rel_interval,
            buildbot_interval_mins: bb_interval,
            post_comments: post_comments,
        })

    } else {

        Err(vars.iter()
            .filter(|&(_, v)| v.is_err())
            .map(|(&k, _)| k)
            .collect())

    }
}

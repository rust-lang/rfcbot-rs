// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std::collections::BTreeMap;
use std::env;

lazy_static! {
    pub static ref CONFIG: Config = {
        match init() {
            Ok(c) => c,
            Err(missing) => panic!("Unable to load environment variables: {:?}", missing),
        }
    };
}

#[derive(Debug)]
pub struct Config {
    pub db_url: String,
    pub db_pool_size: u32,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_user_agent: String,
}

const DB_URL: &'static str = "DATABASE_URL";
const DB_POOL_SIZE: &'static str = "DATABASE_POOL_SIZE";
const GITHUB_ID: &'static str = "GITHUB_CLIENT_ID";
const GITHUB_SECRET: &'static str = "GITHUB_CLIENT_SECRET";
const GITHUB_UA: &'static str = "GITHUB_USER_AGENT";

// this is complex, but we'll shortly need a lot more config items
// so checking them automagically seems like a nice solution
pub fn init() -> Result<Config, Vec<&'static str>> {

    let mut vars: BTreeMap<&'static str, Result<String, _>> = BTreeMap::new();
    let keys = vec![DB_URL, DB_POOL_SIZE, GITHUB_ID, GITHUB_SECRET, GITHUB_UA];

    for var in keys.into_iter() {
        vars.insert(var, env::var(var));
    }

    let all_found = vars.iter().all(|(_, v)| v.is_ok());
    if all_found {
        let mut vars = vars.into_iter()
                           .map(|(k, v)| (k, v.unwrap()))
                           .collect::<BTreeMap<_, _>>();

        let db_url = vars.remove(DB_URL).unwrap();
        let db_pool_size = vars.remove(DB_POOL_SIZE).unwrap();
        let db_pool_size = match db_pool_size.parse::<u32>() {
            Ok(size) => size,
            Err(_) => return Err(vec![DB_POOL_SIZE]),
        };

        let gh_id = vars.remove(GITHUB_ID).unwrap();
        let gh_secret = vars.remove(GITHUB_SECRET).unwrap();
        let gh_ua = vars.remove(GITHUB_UA).unwrap();

        Ok(Config {
            db_url: db_url,
            db_pool_size: db_pool_size,
            github_client_id: gh_id,
            github_client_secret: gh_secret,
            github_user_agent: gh_ua,
        })

    } else {

        Err(vars.iter()
                .filter(|&(_, v)| v.is_err())
                .map(|(&k, _)| k)
                .collect())

    }
}

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
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_user_agent: String,
}

const GITHUB_ID: &'static str = "GITHUB_CLIENT_ID";
const GITHUB_SECRET: &'static str = "GITHUB_CLIENT_SECRET";
const GITHUB_UA: &'static str = "GITHUB_USER_AGENT";

// this is complex, but we'll shortly need a lot more config items
// so checking them automagically seems like a nice solution
pub fn init() -> Result<Config, Vec<&'static str>> {

    let mut variables: BTreeMap<&'static str, Result<String, _>> = BTreeMap::new();
    let keys = vec![GITHUB_ID, GITHUB_SECRET, GITHUB_UA];

    for var in keys.into_iter() {
        variables.insert(var, env::var(var));
    }

    let all_found = variables.iter().all(|(_, v)| v.is_ok());
    if all_found {

        let gh_id = variables.remove(GITHUB_ID).unwrap().unwrap();
        let gh_secret = variables.remove(GITHUB_SECRET).unwrap().unwrap();
        let gh_ua = variables.remove(GITHUB_UA).unwrap().unwrap();

        Ok(Config {
            github_client_id: gh_id,
            github_client_secret: gh_secret,
            github_user_agent: gh_ua,
        })

    } else {

        Err(variables.iter()
                     .filter(|&(_, v)| v.is_err())
                     .map(|(&k, _)| k)
                     .collect())

    }
}

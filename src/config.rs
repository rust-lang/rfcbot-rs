// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std::collections::BTreeMap;
use std::env;

#[derive(Debug)]
pub struct Config {
    github_client_id: String,
    github_client_secret: String,
}

const GITHUB_CLIENT_ID: &'static str = "GITHUB_CLIENT_ID";
const GITHUB_CLIENT_SECRET: &'static str = "GITHUB_CLIENT_SECRET";

// this is complex, but we'll shortly need a lot more config items
// so checking them automagically seems like a nice solution
pub fn init() -> Result<Config, Vec<&'static str>> {

    let mut variables: BTreeMap<&'static str, Result<String, _>> = BTreeMap::new();
    let keys = vec![GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET];

    for var in keys.into_iter() {
        variables.insert(var, env::var(var));
    }

    let all_found = variables.iter().all(|(_, v)| v.is_ok());
    if all_found {

        let gh_id = variables.remove(GITHUB_CLIENT_ID).unwrap().unwrap();
        let gh_secret = variables.remove(GITHUB_CLIENT_SECRET).unwrap().unwrap();

        Ok(Config {
            github_client_id: gh_id,
            github_client_secret: gh_secret,
        })

    } else {

        Err(variables.iter()
                     .filter(|&(_, v)| v.is_err())
                     .map(|(&k, _)| k)
                     .collect())

    }
}

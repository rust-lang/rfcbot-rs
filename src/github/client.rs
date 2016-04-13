use std::collections::BTreeMap;
use std::io::Read;

use hyper;
use hyper::client::{RedirectPolicy, Response};
use hyper::header::Connection;

use config::Config;
use github::error::GitHubResult;

const BASE_URL: &'static str = "https://api.github.com";

header! { (UA, "User-Agent") => [String] }

pub struct Client {
    id: String,
    secret: String,
    ua: String,
    client: hyper::Client,
}

impl Client {
    pub fn from(c: &Config) -> GitHubResult<Self> {
        let mut client = hyper::Client::new();
        client.set_redirect_policy(RedirectPolicy::FollowAll);

        let mut ghc = Client {
            id: c.github_client_id.clone(),
            secret: c.github_client_secret.clone(),
            ua: c.github_user_agent.clone(),
            client: client,
        };

        try!(ghc.check_usage());
        Ok(ghc)
    }

    fn request<'a>(&mut self,
                   end_point: &'a str,
                   params: Option<BTreeMap<String, String>>)
                   -> hyper::error::Result<Response> {

        // TODO(adam) check rate limit and return error if exceeded

        let url = format!("{}/{}?{}", BASE_URL, end_point, self.query_params(params));

        self.client
            .get(&url)
            .header(UA(self.ua.clone()))
            .header(Connection::close())
            .send()

        // TODO(adam) check for failed b/c of rate limit and reset struct fields
    }

    fn query_params(&self, extras: Option<BTreeMap<String, String>>) -> String {
        let mut qp = format!("client_id={}&client_secret={}", self.id, self.secret);

        match extras {
            Some(e) => {
                for (k, v) in e.into_iter() {
                    qp.push_str(&format!("&{}={}", k, v));
                }
            }
            None => (),
        }

        qp
    }

    fn check_usage(&mut self) -> GitHubResult<()> {
        let mut response = try!(self.request("rate_limit", None));

        let mut buf = String::new();
        try!(response.read_to_string(&mut buf));

        // TODO(adam) parse out the response and set what the remaining rate limit is

        Ok(())
    }
}

use std::collections::BTreeMap;
use std::io::Read;
use std::u32;

use chrono::{DateTime, UTC};
use hyper;
use hyper::client::{RedirectPolicy, Response};
use hyper::header::Headers;
use serde::Deserialize;
use serde_json;

use config::Config;
use github::error::GitHubResult;
use github::models::{CommentFromJson, IssueFromJson};

pub const BASE_URL: &'static str = "https://api.github.com";
pub const REPO_OWNER: &'static str = "rust-lang";
pub const REPO: &'static str = "rust";

type ParameterMap = BTreeMap<&'static str, String>;

header! { (UA, "User-Agent") => [String] }
header! { (TZ, "Time-Zone") => [String] }
header! { (Accept, "Accept") => [String] }
header! { (RateLimitRemaining, "X-RateLimit-Remaining") => [u32] }
header! { (RateLimitReset, "X-RateLimit-Reset") => [i64] }
header! { (Link, "Link") => [String] }

const PER_PAGE: u32 = 100;

#[derive(Debug)]
pub struct Client {
    id: String,
    secret: String,
    ua: String,
    client: hyper::Client,
    rate_limit: u32,
    rate_limit_timeout: DateTime<UTC>,
}

impl Client {
    pub fn from(c: &Config) -> GitHubResult<Self> {
        let mut client = hyper::Client::new();
        client.set_redirect_policy(RedirectPolicy::FollowAll);

        Ok(Client {
            id: c.github_client_id.clone(),
            secret: c.github_client_secret.clone(),
            ua: c.github_user_agent.clone(),
            client: client,
            rate_limit: u32::MAX,
            rate_limit_timeout: UTC::now(),
        })
    }

    pub fn issues_since(&self, start: DateTime<UTC>) -> GitHubResult<Vec<IssueFromJson>> {

        let url = format!("{}/repos/{}/{}/issues", BASE_URL, REPO_OWNER, REPO);
        let mut params = BTreeMap::new();

        params.insert("state", "all".to_string());
        params.insert("since", format!("{:?}", start));
        params.insert("state", "all".to_string());
        params.insert("per_page", format!("{}", PER_PAGE));
        params.insert("direction", "asc".to_string());

        // make the request
        self.models_since(&url, &params)
    }

    pub fn comments_since(&self, start: DateTime<UTC>) -> GitHubResult<Vec<CommentFromJson>> {
        let url = format!("{}/repos/{}/{}/issues/comments", BASE_URL, REPO_OWNER, REPO);
        let mut params = BTreeMap::new();

        params.insert("sort", "created".to_string());
        params.insert("direction", "asc".to_string());
        params.insert("since", format!("{:?}", start));
        params.insert("per_page", format!("{}", PER_PAGE));

        self.models_since(&url, &params)
    }

    fn models_since<M: Deserialize>(&self,
                                    start_url: &str,
                                    params: &ParameterMap)
                                    -> GitHubResult<Vec<M>> {
        let mut res = try!(self.request(start_url, true, Some(&params)));

        // let's try deserializing!
        let mut buf = String::new();
        try!(res.read_to_string(&mut buf));

        let mut models = try!(serde_json::from_str::<Vec<M>>(&buf));

        let mut next_url = Self::next_page(&res.headers);
        while next_url.is_some() {
            let url = next_url.unwrap();
            let mut next_res = try!(self.request(&url, false, None));

            buf.clear();
            try!(next_res.read_to_string(&mut buf));

            models.extend(try!(serde_json::from_str::<Vec<M>>(&buf)));

            next_url = Self::next_page(&next_res.headers);
        }

        Ok(models)
    }

    fn next_page(h: &Headers) -> Option<String> {
        if let Some(lh) = h.get::<Link>() {
            for link in (**lh).split(",").map(|s| s.trim()) {

                let tokens = link.split(";").map(|s| s.trim()).collect::<Vec<_>>();

                if tokens.len() != 2 {
                    continue;
                }

                if tokens[1] == "rel=\"next\"" {
                    let url = tokens[0].trim_left_matches('<').trim_right_matches('>').to_string();
                    return Some(url);
                }
            }
        }

        None
    }

    fn request<'a>(&self,
                   url: &'a str,
                   add_auth: bool,
                   params: Option<&ParameterMap>)
                   -> Result<Response, hyper::error::Error> {

        let query_params = self.query_params(add_auth, params);
        let url = if query_params.len() > 0 {
            format!("{}?{}", url, query_params)
        } else {
            String::from(url)
        };

        self.client
            .get(&url)
            .header(UA(self.ua.clone()))
            .header(TZ("UTC".to_string()))
            .header(Accept("application/vnd.github.v3".to_string()))
            .send()
    }

    fn query_params(&self, add_auth: bool, extras: Option<&ParameterMap>) -> String {

        let mut qp = if add_auth {
            format!("client_id={}&client_secret={}", self.id, self.secret)
        } else {
            String::new()
        };

        match extras {
            Some(e) => {
                for (k, v) in e.iter() {
                    if qp.len() > 0 {
                        qp.push('&');
                    }
                    qp.push_str(&format!("{}={}", k, v));
                }
            }
            None => (),
        }

        qp
    }
}

// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std::collections::BTreeMap;
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;
use std::u32;

use chrono::{DateTime, Utc};
use hyper;
use hyper::client::{RedirectPolicy, RequestBuilder, Response};
use hyper::header::{Headers, Authorization, UserAgent};
use hyper::net::HttpsConnector;
use hyper::status::StatusCode;
use hyper_native_tls::NativeTlsClient;
use serde::de::DeserializeOwned;
use serde_json;

use config::CONFIG;
use error::{DashError, DashResult};
use github::models::{CommentFromJson, IssueFromJson, PullRequestFromJson, PullRequestUrls};

pub const BASE_URL: &'static str = "https://api.github.com";

pub const DELAY: u64 = 300;

type ParameterMap = BTreeMap<&'static str, String>;

header! { (TZ, "Time-Zone") => [String] }
header! { (Accept, "Accept") => [String] }
header! { (RateLimitRemaining, "X-RateLimit-Remaining") => [u32] }
header! { (RateLimitReset, "X-RateLimit-Reset") => [i64] }
header! { (Link, "Link") => [String] }

const PER_PAGE: u32 = 100;

#[derive(Debug)]
pub struct Client {
    token: String,
    ua: String,
    client: hyper::Client,
    rate_limit: u32,
    rate_limit_timeout: DateTime<Utc>,
}

fn read_to_string<R: Read>(reader: &mut R) -> DashResult<String> {    
    let mut string = String::new();
    reader.read_to_string(&mut string)?;
    Ok(string)
}

impl Client {
    pub fn new() -> Self {
        let tls_connector = HttpsConnector::new(NativeTlsClient::new().unwrap());
        let mut client = hyper::Client::with_connector(tls_connector);
        client.set_redirect_policy(RedirectPolicy::FollowAll);

        Client {
            token: CONFIG.github_access_token.clone(),
            ua: CONFIG.github_user_agent.clone(),
            client: client,
            rate_limit: u32::MAX,
            rate_limit_timeout: Utc::now(),
        }
    }

    pub fn org_repos(&self, org: &str) -> DashResult<Vec<String>> {
        let url = format!("{}/orgs/{}/repos", BASE_URL, org);
        let vals: Vec<serde_json::Value> = self.get_models(&url, None)?;

        let mut repos = Vec::new();
        for v in vals {
            if let Some(v) = v.as_object() {
                if let Some(n) = v.get("name") {
                    if let Some(s) = n.as_str() {
                        repos.push(format!("{}/{}", org, s));
                        continue;
                    }
                }
            }
            throw!(DashError::Misc(None))

        }
        Ok(repos)
    }

    pub fn issues_since(&self, repo: &str, start: DateTime<Utc>) -> DashResult<Vec<IssueFromJson>> {
        self.get_models(&format!("{}/repos/{}/issues", BASE_URL, repo),
            Some(&btreemap! {
                "state" => "all".to_string(),
                "since" => format!("{:?}", start),
                "per_page" => format!("{}", PER_PAGE),
                "direction" => "asc".to_string()    
            }))
    }

    pub fn comments_since(&self,
                          repo: &str,
                          start: DateTime<Utc>)
                          -> DashResult<Vec<CommentFromJson>> {
        self.get_models(&format!("{}/repos/{}/issues/comments", BASE_URL, repo),
            Some(&btreemap! {
                "sort" => "created".to_string(),
                "direction" => "asc".to_string(),
                "since" => format!("{:?}", start),
                "per_page" => format!("{}", PER_PAGE)
            }))
    }

    fn get_models<M: DeserializeOwned>(&self,
                                       start_url: &str,
                                       params: Option<&ParameterMap>)
                                       -> DashResult<Vec<M>> {

        let mut res = self.get(start_url, params)?;
        let mut models = self.deserialize::<Vec<M>>(&mut res)?;
        while let Some(url) = Self::next_page(&res.headers) {
            sleep(Duration::from_millis(DELAY));
            res = self.get(&url, None)?;
            models.extend(self.deserialize::<Vec<M>>(&mut res)?);
        }
        Ok(models)
    }

    pub fn fetch_pull_request(&self, pr_info: &PullRequestUrls) -> DashResult<PullRequestFromJson> {
        if let Some(url) = pr_info.get("url") {
            let mut res = self.get(url, None)?;
            self.deserialize(&mut res)
        } else {
            throw!(DashError::Misc(None))
        }
    }

    fn next_page(h: &Headers) -> Option<String> {
        if let Some(lh) = h.get::<Link>() {
            for link in (**lh).split(',').map(|s| s.trim()) {

                let tokens = link.split(';').map(|s| s.trim()).collect::<Vec<_>>();

                if tokens.len() != 2 {
                    continue;
                }

                if tokens[1] == "rel=\"next\"" {
                    let url = tokens[0]
                        .trim_left_matches('<')
                        .trim_right_matches('>')
                        .to_string();
                    return Some(url);
                }
            }
        }

        None
    }

    pub fn close_issue(&self, repo: &str, issue_num: i32) -> DashResult<()> {
        let url = format!("{}/repos/{}/issues/{}", BASE_URL, repo, issue_num);
        let payload = serde_json::to_string(&btreemap!("state" => "closed"))?;
        let mut res = self.patch(&url, &payload)?;

        if StatusCode::Ok != res.status {
            throw!(DashError::Misc(Some(read_to_string(&mut res)?)))
        }

        Ok(())
    }

    pub fn add_label(&self, repo: &str, issue_num: i32, label: &str) -> DashResult<()> {
        let url = format!("{}/repos/{}/issues/{}/labels", BASE_URL, repo, issue_num);
        let payload = serde_json::to_string(&[label])?;

        let mut res = self.post(&url, &payload)?;

        if StatusCode::Ok != res.status {
            throw!(DashError::Misc(Some(read_to_string(&mut res)?)))
        }

        Ok(())
    }

    pub fn remove_label(&self, repo: &str, issue_num: i32, label: &str) -> DashResult<()> {
        let url = format!("{}/repos/{}/issues/{}/labels/{}",
                          BASE_URL,
                          repo,
                          issue_num,
                          label);
        let mut res = self.delete(&url)?;

        if StatusCode::NoContent != res.status {
            throw!(DashError::Misc(Some(read_to_string(&mut res)?)))
        }

        Ok(())
    }

    pub fn new_comment(&self,
                       repo: &str,
                       issue_num: i32,
                       text: &str)
                       -> DashResult<CommentFromJson> {
        let url = format!("{}/repos/{}/issues/{}/comments", BASE_URL, repo, issue_num);
        let payload = serde_json::to_string(&btreemap!("body" => text))?;
        // FIXME propagate an error if it's a 404 or other error
        self.deserialize(&mut self.post(&url, &payload)?)
    }

    pub fn edit_comment(&self,
                        repo: &str,
                        comment_num: i32,
                        text: &str)
                        -> DashResult<CommentFromJson> {
        let url = format!("{}/repos/{}/issues/comments/{}",
                          BASE_URL,
                          repo,
                          comment_num);

        let payload = serde_json::to_string(&btreemap!("body" => text))?;

        // FIXME propagate an error if it's a 404 or other error
        self.deserialize(&mut self.patch(&url, &payload)?)
    }

    fn patch(&self, url: &str, payload: &str) -> Result<Response, hyper::error::Error> {
        self.set_headers(self.client.patch(url).body(payload))
            .send()
    }

    fn post(&self, url: &str, payload: &str) -> Result<Response, hyper::error::Error> {
        self.set_headers(self.client.post(url).body(payload)).send()
    }

    fn delete(&self, url: &str) -> Result<Response, hyper::error::Error> {
        self.set_headers(self.client.delete(url)).send()
    }

    fn get(&self,
           url: &str,
           params: Option<&ParameterMap>)
           -> Result<Response, hyper::error::Error> {
        let qp_string = match params {
            Some(p) => {
                let mut qp = String::from("?");
                for (k, v) in p {
                    if qp.len() > 1 {
                        qp.push('&');
                    }
                    qp.push_str(&format!("{}={}", k, v));
                }
                qp
            }
            None => "".to_string(),
        };

        let url = format!("{}{}", url, qp_string);

        debug!("GETing: {}", &url);

        self.set_headers(self.client.get(&url)).send()
    }

    fn deserialize<M: DeserializeOwned>(&self, res: &mut Response) -> DashResult<M> {
        let mut buf = String::new();
        res.read_to_string(&mut buf)?;

        match serde_json::from_str(&buf) {
            Ok(m) => Ok(m),
            Err(why) => {
                error!("Unable to parse from JSON ({:?}): {}", why, buf);
                throw!(why)
            }
        }
    }

    fn set_headers<'a>(&self, req: RequestBuilder<'a>) -> RequestBuilder<'a> {
        req.header(Authorization(format!("token {}", &self.token)))
            .header(UserAgent(self.ua.clone()))
            .header(TZ("UTC".to_string()))
            .header(Accept("application/vnd.github.v3".to_string()))
            .header(hyper::header::Connection::close())
    }
}

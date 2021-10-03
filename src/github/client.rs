// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std::collections::BTreeMap;
use std::thread::sleep;
use std::time::Duration;
use std::u32;

use chrono::{DateTime, Utc};
use reqwest::{self, header::HeaderMap, Response, StatusCode};
use serde::de::DeserializeOwned;

use crate::config::CONFIG;
use crate::domain::github::GitHubUser;
use crate::error::{DashError, DashResult};
use crate::github::models::{CommentFromJson, IssueFromJson, PullRequestFromJson, PullRequestUrls};

pub const BASE_URL: &str = "https://api.github.com";

pub const DELAY: u64 = 300;

type ParameterMap = BTreeMap<&'static str, String>;

const PER_PAGE: u32 = 100;

#[derive(Debug)]
pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        if !CONFIG.github_access_token.trim().is_empty() {
            headers.insert(
                "Authorization",
                format!("token {}", CONFIG.github_access_token)
                    .parse()
                    .unwrap(),
            );
        }
        headers.insert("User-Agent", CONFIG.github_user_agent.parse().unwrap());
        headers.insert("Time-Zone", "UTC".parse().unwrap());
        headers.insert("Accept", "application/vnd.github.v3".parse().unwrap());
        headers.insert("Connection", "close".parse().unwrap());
        Client {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
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
        self.get_models(
            &format!("{}/repos/{}/issues", BASE_URL, repo),
            Some(&btreemap! {
                "state" => "all".to_string(),
                "since" => format!("{:?}", start),
                "per_page" => format!("{}", PER_PAGE),
                "direction" => "asc".to_string()
            }),
        )
    }

    pub fn comments_since(
        &self,
        repo: &str,
        start: DateTime<Utc>,
    ) -> DashResult<Vec<CommentFromJson>> {
        self.get_models(
            &format!("{}/repos/{}/issues/comments", BASE_URL, repo),
            Some(&btreemap! {
                "sort" => "created".to_string(),
                "direction" => "asc".to_string(),
                "since" => format!("{:?}", start),
                "per_page" => format!("{}", PER_PAGE)
            }),
        )
    }

    fn get_models<M: DeserializeOwned>(
        &self,
        start_url: &str,
        params: Option<&ParameterMap>,
    ) -> DashResult<Vec<M>> {
        let mut res = self.get(start_url, params)?;
        let mut models: Vec<M> = res.json()?;
        while let Some(url) = Self::next_page(res.headers()) {
            sleep(Duration::from_millis(DELAY));
            res = self.get(&url, None)?;
            models.extend(res.json::<Vec<M>>()?);
        }
        Ok(models)
    }

    pub fn fetch_pull_request(&self, pr_info: &PullRequestUrls) -> DashResult<PullRequestFromJson> {
        if let Some(url) = pr_info.get("url") {
            Ok(self.get(url, None)?.json()?)
        } else {
            throw!(DashError::Misc(None))
        }
    }

    fn next_page(h: &HeaderMap) -> Option<String> {
        if let Some(lh) = h.get("Link") {
            let lh = &lh.to_str().unwrap();
            for link in (**lh).split(',').map(|s| s.trim()) {
                let tokens = link.split(';').map(str::trim).collect::<Vec<_>>();

                if tokens.len() != 2 {
                    continue;
                }

                if tokens[1] == "rel=\"next\"" {
                    let url = tokens[0]
                        .trim_start_matches('<')
                        .trim_end_matches('>')
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

        if StatusCode::OK != res.status() {
            throw!(DashError::Misc(Some(res.text()?)))
        }

        Ok(())
    }

    pub fn add_label(&self, repo: &str, issue_num: i32, label: &str) -> DashResult<()> {
        let url = format!("{}/repos/{}/issues/{}/labels", BASE_URL, repo, issue_num);
        let payload = serde_json::to_string(&[label])?;

        let mut res = self.post(&url, &payload)?;

        if StatusCode::OK != res.status() {
            throw!(DashError::Misc(Some(res.text()?)))
        }

        Ok(())
    }

    pub fn remove_label(&self, repo: &str, issue_num: i32, label: &str) -> DashResult<()> {
        let url = format!(
            "{}/repos/{}/issues/{}/labels/{}",
            BASE_URL, repo, issue_num, label
        );
        let mut res = self.delete(&url)?;

        if StatusCode::NO_CONTENT != res.status() {
            throw!(DashError::Misc(Some(res.text()?)))
        }

        Ok(())
    }

    pub fn new_comment(
        &self,
        repo: &str,
        issue_num: i32,
        text: &str,
    ) -> DashResult<CommentFromJson> {
        let url = format!("{}/repos/{}/issues/{}/comments", BASE_URL, repo, issue_num);
        let payload = serde_json::to_string(&btreemap!("body" => text))?;
        Ok(self.post(&url, &payload)?.error_for_status()?.json()?)
    }

    pub fn edit_comment(
        &self,
        repo: &str,
        comment_num: i32,
        text: &str,
    ) -> DashResult<CommentFromJson> {
        let url = format!(
            "{}/repos/{}/issues/comments/{}",
            BASE_URL, repo, comment_num
        );
        let payload = serde_json::to_string(&btreemap!("body" => text))?;
        Ok(self.patch(&url, &payload)?.error_for_status()?.json()?)
    }

    pub fn get_user(&self, name: &str) -> DashResult<GitHubUser> {
        let url = format!("{}/users/{}", BASE_URL, name);
        Ok(self.get(&url, None)?.error_for_status()?.json()?)
    }

    fn patch(&self, url: &str, payload: &str) -> Result<Response, reqwest::Error> {
        self.client.patch(url).body(payload.to_string()).send()
    }

    fn post(&self, url: &str, payload: &str) -> Result<Response, reqwest::Error> {
        self.client.post(url).body(payload.to_string()).send()
    }

    fn delete(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.client.delete(url).send()
    }

    fn get(&self, url: &str, params: Option<&ParameterMap>) -> Result<Response, reqwest::Error> {
        debug!("GETing: {}", &url);
        let mut builder = self.client.get(url);
        if let Some(params) = params {
            builder = builder.query(params);
        }
        builder.send()
    }
}

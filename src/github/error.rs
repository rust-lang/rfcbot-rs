// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std;
use std::convert::From;
use std::io;

use diesel;
use hyper;
use r2d2;
use serde_json;

pub type GitHubResult<T> = std::result::Result<T, GitHubError>;

#[derive(Debug)]
pub enum GitHubError {
    Hyper(hyper::error::Error),
    Io(io::Error),
    Serde(serde_json::error::Error),
    R2d2Timeout(r2d2::GetTimeout),
    DieselError(diesel::result::Error),
    Misc,
}

impl From<hyper::error::Error> for GitHubError {
    fn from(e: hyper::error::Error) -> Self {
        GitHubError::Hyper(e)
    }
}

impl From<io::Error> for GitHubError {
    fn from(e: io::Error) -> Self {
        GitHubError::Io(e)
    }
}

impl From<serde_json::error::Error> for GitHubError {
    fn from(e: serde_json::error::Error) -> Self {
        GitHubError::Serde(e)
    }
}

impl From<r2d2::GetTimeout> for GitHubError {
    fn from(e: r2d2::GetTimeout) -> Self {
        GitHubError::R2d2Timeout(e)
    }
}

impl From<diesel::result::Error> for GitHubError {
    fn from(e: diesel::result::Error) -> Self {
        GitHubError::DieselError(e)
    }
}

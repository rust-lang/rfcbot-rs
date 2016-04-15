use std;
use std::convert::From;
use std::io;

use hyper;
use serde_json;

pub type GitHubResult<T> = std::result::Result<T, GitHubError>;

#[derive(Debug)]
pub enum GitHubError {
    Hyper(hyper::error::Error),
    Io(io::Error),
    Serde(serde_json::error::Error),
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

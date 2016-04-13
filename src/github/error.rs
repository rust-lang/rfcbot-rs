use std;
use std::convert::From;
use std::io;

use chrono::{DateTime, UTC};
use hyper;

pub type GitHubResult<T> = std::result::Result<T, GitHubError>;

pub enum GitHubError {
    Hyper(hyper::error::Error),
    Io(io::Error),
    RateLimit(DateTime<UTC>),
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

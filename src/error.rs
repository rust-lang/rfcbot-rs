// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std::convert::From;
use std::io;

use rocket_contrib::templates::handlebars;

pub type DashResult<T> = std::result::Result<T, DashError>;

#[derive(Debug)]
pub enum DashError {
    Reqwest(reqwest::Error),
    Io(io::Error),
    Serde(serde_json::error::Error),
    R2d2(diesel::r2d2::PoolError),
    DieselError(diesel::result::Error),
    Template(handlebars::RenderError),
    Misc(Option<String>),
}

impl From<handlebars::RenderError> for DashError {
    fn from(e: handlebars::RenderError) -> Self { DashError::Template(e) }
}

impl From<reqwest::Error> for DashError {
    fn from(e: reqwest::Error) -> Self { DashError::Reqwest(e) }
}

impl From<io::Error> for DashError {
    fn from(e: io::Error) -> Self { DashError::Io(e) }
}

impl From<serde_json::error::Error> for DashError {
    fn from(e: serde_json::error::Error) -> Self { DashError::Serde(e) }
}

impl From<diesel::r2d2::PoolError> for DashError {
    fn from(e: diesel::r2d2::PoolError) -> Self { DashError::R2d2(e) }
}

impl From<diesel::result::Error> for DashError {
    fn from(e: diesel::result::Error) -> Self { DashError::DieselError(e) }
}

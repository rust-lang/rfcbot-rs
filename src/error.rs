// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std;
use std::convert::From;
use std::io;

use diesel;
use hyper;
use iron;
use r2d2;
use serde_json;

pub type DashResult<T> = std::result::Result<T, DashError>;

#[derive(Debug)]
pub enum DashError {
    Hyper(hyper::error::Error),
    Io(io::Error),
    Serde(serde_json::error::Error),
    R2d2Timeout(r2d2::GetTimeout),
    DieselError(diesel::result::Error),
    Misc,
}

impl From<hyper::error::Error> for DashError {
    fn from(e: hyper::error::Error) -> Self { DashError::Hyper(e) }
}

impl From<io::Error> for DashError {
    fn from(e: io::Error) -> Self { DashError::Io(e) }
}

impl From<serde_json::error::Error> for DashError {
    fn from(e: serde_json::error::Error) -> Self { DashError::Serde(e) }
}

impl From<r2d2::GetTimeout> for DashError {
    fn from(e: r2d2::GetTimeout) -> Self { DashError::R2d2Timeout(e) }
}

impl From<diesel::result::Error> for DashError {
    fn from(e: diesel::result::Error) -> Self { DashError::DieselError(e) }
}

impl From<DashError> for iron::IronError {
    fn from(e: DashError) -> iron::IronError {
        iron::IronError {
            error: match e {
                DashError::Hyper(e) => Box::new(e),
                DashError::Io(e) => Box::new(e),
                DashError::Serde(e) => Box::new(e),
                DashError::R2d2Timeout(e) => Box::new(e),
                DashError::DieselError(e) => Box::new(e),
                DashError::Misc => {
                    Box::new(io::Error::new(io::ErrorKind::Other, "miscellaneous error"))
                }
            },
            response: iron::Response::with(iron::status::InternalServerError),
        }
    }
}

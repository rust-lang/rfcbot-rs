use chrono::{Duration, UTC};
use iron::method::Method;
use iron::prelude::*;
use iron::status;
use serde_json::ser;

use error::DashError;
use reports;

pub fn default_summary(req: &mut Request) -> IronResult<Response> {
    match req.method {
        Method::Get => {

            let today = UTC::today().naive_utc();
            let since = today - Duration::days(30);

            let summary = try!(reports::summary(since, today));
            let summary_json = try!(ser::to_string(&summary).map_err(|e| {
                let e: DashError = e.into();
                e
            }));

            Ok(Response::with((status::Ok, summary_json)))
        }
        _ => Ok(Response::with(status::Unauthorized)),
    }
}

pub fn summary(req: &mut Request) -> IronResult<Response> {
    match req.method {
        Method::Get => {

            // TODO parse the request dates out of the url

            unimplemented!();
        }
        _ => Ok(Response::with(status::Unauthorized)),
    }
}

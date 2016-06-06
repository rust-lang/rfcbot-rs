use std::error::Error;
use std::fmt;

use chrono::{NaiveDate, Duration, UTC};
use iron::prelude::*;
use iron::status;
use serde_json::ser;
use urlencoded::{UrlDecodingError, UrlEncodedQuery};

use error::DashError;
use reports;

const DATE_FORMAT: &'static str = "%Y%m%d";

macro_rules! make_dated_endpoint {
    ($f:ident, $g:ident) => {
        pub fn $f(req: &mut Request) -> IronResult<Response> {

            let (since, until) = match parse_dates_from_query(req) {
                Ok((s, u)) => (s, u),
                Err(why) => return Ok(Response::with((status::BadRequest, why.description())))
            };

            let summary = try!(reports::$g(since, until));
            let summary_json = try!(ser::to_string(&summary).map_err(|e| {
                let e: DashError = e.into();
                e
            }));

            Ok(Response::with((status::Ok, summary_json)))
        }
    }
}

make_dated_endpoint!(pull_requests, pr_summary);
make_dated_endpoint!(issues, issue_summary);
make_dated_endpoint!(buildbots, ci_summary);
make_dated_endpoint!(releases, release_summary);

fn parse_dates_from_query(req: &mut Request) -> IronResult<(NaiveDate, NaiveDate)> {
    let today = UTC::today().naive_utc();
    let since = today - Duration::days(30);

    let default = Ok((since, today));
    let errmsg = "Invalid query string".to_string();

    match req.get_ref::<UrlEncodedQuery>() {
        Ok(params) => {
            // if the query string is empty, it'll be an error, so this is only to check
            // whether it contains one param but not the other
            if !(params.contains_key("start") && params.contains_key("end")) {
                return Err(IronError::new(DateParseError::WrongNumber, errmsg));
            }

            let start = params.get("start").unwrap();
            let end = params.get("end").unwrap();

            if start.len() == 1 && end.len() == 1 {

                let (start, end) = (start.get(0).unwrap(), end.get(0).unwrap());

                let start = match NaiveDate::parse_from_str(&start, DATE_FORMAT) {
                    Ok(s) => s,
                    Err(why) => return Err(IronError::new(why, errmsg)),
                };

                let end = match NaiveDate::parse_from_str(&end, DATE_FORMAT) {
                    Ok(s) => s,
                    Err(why) => return Err(IronError::new(why, errmsg)),
                };

                Ok((start, end))
            } else {
                Err(IronError::new(DateParseError::WrongNumber, errmsg))
            }
        },
        Err(why) => {
            match why {
                UrlDecodingError::BodyError(why) => { Err(IronError::new(why, errmsg)) },
                UrlDecodingError::EmptyQuery => default,
            }
        }
    }
}

#[derive(Debug)]
enum DateParseError {
    WrongNumber,
}

impl fmt::Display for DateParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for DateParseError {
    fn description(&self) -> &str {
        match self {
            &DateParseError::WrongNumber => "Incorrect number of date params",
        }
    }
}

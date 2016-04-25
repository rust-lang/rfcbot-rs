use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate, NaiveDateTime, UTC};
use diesel::expression::dsl::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::types::{BigInt, Date};
use domain::schema::pullrequest::dsl::*;

use DB_POOL;
use error::DashResult;

#[derive(Clone, Debug)]
pub struct DashSummary {
    prs_opened_per_day: BTreeMap<NaiveDate, i64>,
    prs_closed_per_day: BTreeMap<NaiveDate, i64>,
    prs_merged_per_day: BTreeMap<NaiveDate, i64>,
}

pub fn summary() -> DashResult<DashSummary> {
    let conn = try!(DB_POOL.get());

    let until = UTC::now().naive_utc();
    let since = until - Duration::days(90);

    Ok(DashSummary {
        prs_opened_per_day: try!(prs_opened_per_day(since, until, &conn)),
        prs_closed_per_day: try!(prs_closed_per_day(since, until, &conn)),
        prs_merged_per_day: try!(prs_merged_per_day(since, until, &conn)),
    })
}

fn prs_opened_per_day(since: NaiveDateTime,
                      until: NaiveDateTime,
                      conn: &PgConnection)
                      -> DashResult<BTreeMap<NaiveDate, i64>> {
    // TODO (adam) waiting on multiple aggregates in diesel
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("created_at::date as d, COUNT(*)"))
                       .filter(created_at.ge(since).and(created_at.le(until)))
                       .group_by(d)
                       .get_results(conn))
           .into_iter()
           .collect())
}

fn prs_closed_per_day(since: NaiveDateTime,
                      until: NaiveDateTime,
                      conn: &PgConnection)
                      -> DashResult<BTreeMap<NaiveDate, i64>> {
    // TODO (adam) waiting on multiple aggregates in diesel
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("closed_at::date as d, COUNT(*)"))
                       .filter(closed_at.is_not_null()
                                        .and(closed_at.ge(since).and(closed_at.le(until))))
                       .group_by(d)
                       .get_results(conn))
           .into_iter()
           .collect())
}

fn prs_merged_per_day(since: NaiveDateTime,
                      until: NaiveDateTime,
                      conn: &PgConnection)
                      -> DashResult<BTreeMap<NaiveDate, i64>> {
    // TODO (adam) waiting on multiple aggregates in diesel
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("merged_at::date as d, COUNT(*)"))
                       .filter(merged_at.is_not_null()
                                        .and(merged_at.ge(since).and(merged_at.le(until))))
                       .group_by(d)
                       .get_results(conn))
           .into_iter()
           .collect())
}

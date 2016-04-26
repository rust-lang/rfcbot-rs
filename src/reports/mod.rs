use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate, NaiveDateTime, UTC};
use diesel::expression::dsl::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::types::{BigInt, Date, Double, Text};
use domain::schema::pullrequest::dsl::*;

use DB_POOL;
use error::DashResult;

#[derive(Clone, Debug)]
pub struct DashSummary {
    prs_opened_per_day: BTreeMap<NaiveDate, i64>,
    prs_closed_per_day: BTreeMap<NaiveDate, i64>,
    prs_merged_per_day: BTreeMap<NaiveDate, i64>,
    prs_num_closed_per_week: BTreeMap<NaiveDate, i64>,
    prs_days_open_before_close: BTreeMap<NaiveDate, f64>,
    prs_current_open_age_days: f64,
}

pub fn summary() -> DashResult<DashSummary> {
    let conn = try!(DB_POOL.get());

    let until = UTC::now().naive_utc();
    let since = until - Duration::days(90);

    let pr_open_time = try!(prs_open_time_before_close(since, until, &conn));

    Ok(DashSummary {
        prs_opened_per_day: try!(prs_opened_per_day(since, until, &conn)),
        prs_closed_per_day: try!(prs_closed_per_day(since, until, &conn)),
        prs_merged_per_day: try!(prs_merged_per_day(since, until, &conn)),
        prs_num_closed_per_week: pr_open_time.0,
        prs_days_open_before_close: pr_open_time.1,
        prs_current_open_age_days: try!(open_prs_avg_days_old(&conn)),
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

fn prs_open_time_before_close
    (since: NaiveDateTime,
     until: NaiveDateTime,
     conn: &PgConnection)
     -> DashResult<(BTreeMap<NaiveDate, i64>, BTreeMap<NaiveDate, f64>)> {

    let w = sql::<Text>("iso_closed_week");
    let triples = try!(pullrequest.select(sql::<(BigInt, Double, Text)>("\
        COUNT(*), \
        \
        AVG(EXTRACT(EPOCH FROM closed_at) - \
          EXTRACT(EPOCH FROM created_at)) \
          / (60 * 60 * 24), \
        \
        EXTRACT(ISOYEAR FROM closed_at)::text || '-' || \
          EXTRACT(WEEK FROM closed_at)::text || '-6' AS iso_closed_week"))
                                  .filter(closed_at.is_not_null()
                                                   .and(closed_at.ge(since))
                                                   .and(closed_at.le(until)))
                                  .group_by(w)
                                  .get_results::<(i64, f64, String)>(conn));

    let mut num_closed_map = BTreeMap::new();
    let mut days_open_map = BTreeMap::new();

    for (num_closed, days_open, week_str) in triples {
        // this will give us the beginning of the week we're describing (I think)
        // unwrapping this parsing is fine since we dictate above what format is acceptable
        let d = NaiveDate::parse_from_str(&week_str, "%G-%V-%w").unwrap();
        num_closed_map.insert(d, num_closed);
        days_open_map.insert(d, days_open);
    }

    Ok((num_closed_map, days_open_map))
}

fn open_prs_avg_days_old(conn: &PgConnection) -> DashResult<f64> {
    Ok(try!(pullrequest.select(sql::<Double>("AVG(EXTRACT(EPOCH FROM (now() - created_at))) / \
                                              (60 * 60 * 24)"))
                       .filter(closed_at.is_null())
                       .first(conn)))
}

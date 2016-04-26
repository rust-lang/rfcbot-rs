use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate, NaiveDateTime, UTC};
use diesel::expression::dsl::*;
use diesel::prelude::*;
use diesel::types::{BigInt, Date, Double, Integer, Text, Timestamp};

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
    prs_bors_retries: BTreeMap<i32, i64>,
}

pub fn summary() -> DashResult<DashSummary> {
    let until = UTC::now().naive_utc();
    let since = until - Duration::days(90);

    let pr_open_time = try!(prs_open_time_before_close(since, until));

    Ok(DashSummary {
        prs_opened_per_day: try!(prs_opened_per_day(since, until)),
        prs_closed_per_day: try!(prs_closed_per_day(since, until)),
        prs_merged_per_day: try!(prs_merged_per_day(since, until)),
        prs_num_closed_per_week: pr_open_time.0,
        prs_days_open_before_close: pr_open_time.1,
        prs_current_open_age_days: try!(open_prs_avg_days_old()),
        prs_bors_retries: try!(bors_retries_per_pr(since, until)),
    })
}

pub fn prs_opened_per_day(since: NaiveDateTime,
                      until: NaiveDateTime)
                      -> DashResult<BTreeMap<NaiveDate, i64>> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("created_at::date as d, COUNT(*)"))
                       .filter(created_at.ge(since))
                       .filter(created_at.le(until))
                       .group_by(d)
                       .get_results(&*conn))
           .into_iter()
           .collect())
}

pub fn prs_closed_per_day(since: NaiveDateTime,
                      until: NaiveDateTime)
                      -> DashResult<BTreeMap<NaiveDate, i64>> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("closed_at::date as d, COUNT(*)"))
                       .filter(closed_at.is_not_null())
                       .filter(closed_at.ge(since))
                       .filter(closed_at.le(until))
                       .group_by(d)
                       .get_results(&*conn))
           .into_iter()
           .collect())
}

pub fn prs_merged_per_day(since: NaiveDateTime,
                      until: NaiveDateTime)
                      -> DashResult<BTreeMap<NaiveDate, i64>> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("merged_at::date as d, COUNT(*)"))
                       .filter(merged_at.is_not_null())
                       .filter(merged_at.ge(since))
                       .filter(merged_at.le(until))
                       .group_by(d)
                       .get_results(&*conn))
           .into_iter()
           .collect())
}

pub fn prs_open_time_before_close
    (since: NaiveDateTime,
     until: NaiveDateTime)
     -> DashResult<(BTreeMap<NaiveDate, i64>, BTreeMap<NaiveDate, f64>)> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());

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
                                  .filter(closed_at.is_not_null())
                                  .filter(closed_at.ge(since))
                                  .filter(closed_at.le(until))
                                  .group_by(w)
                                  .get_results::<(i64, f64, String)>(&*conn));

    let mut num_closed_map = BTreeMap::new();
    let mut days_open_map = BTreeMap::new();

    for (num_closed, days_open, week_str) in triples {
        // this will give us the end of each week we're describing
        // unwrapping this parsing is fine since we dictate above what format is acceptable
        let d = NaiveDate::parse_from_str(&week_str, "%G-%V-%w").unwrap();
        num_closed_map.insert(d, num_closed);
        days_open_map.insert(d, days_open);
    }

    Ok((num_closed_map, days_open_map))
}

sql_function!(date_part, date_part_t, (part: Text, date: Timestamp) -> Double);

pub fn open_prs_avg_days_old() -> DashResult<f64> {
    use domain::schema::pullrequest::dsl::*;
    let conn = try!(DB_POOL.get());
    Ok(try!(pullrequest.select(sql::<Double>("AVG(EXTRACT(EPOCH FROM (now() - created_at))) / \
                                              (60 * 60 * 24)"))
                       .filter(closed_at.is_null())
                       .first(&*conn)))
}

pub fn bors_retries_per_pr(since: NaiveDateTime,
                       until: NaiveDateTime)
                       -> DashResult<BTreeMap<i32, i64>> {

    use domain::schema::issuecomment::dsl::*;
    let conn = try!(DB_POOL.get());

    Ok(try!(issuecomment.select(sql::<(Integer, BigInt)>("fk_issue, COUNT(*)"))
                        .filter(body.like("%@bors%retry%"))
                        .filter(created_at.ge(since))
                        .filter(created_at.le(until))
                        .group_by(fk_issue)
                        .load(&*conn))
           .into_iter()
           .collect())
}

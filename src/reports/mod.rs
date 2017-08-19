use chrono::{Duration, NaiveDate, NaiveDateTime};
use diesel::expression::dsl::*;
use diesel::prelude::*;
use diesel::select;
use diesel::types::{BigInt, Bool, Date, Double, Integer, Text, VarChar};

use DB_POOL;
use domain::releases::Release;
use error::DashResult;

pub mod nag;

pub type EpochTimestamp = i64;

#[derive(Clone, Debug, Serialize)]
pub struct PullRequestSummary {
    opened_per_day: Vec<(EpochTimestamp, i64)>,
    closed_per_day: Vec<(EpochTimestamp, i64)>,
    merged_per_day: Vec<(EpochTimestamp, i64)>,
    days_open_before_close: Vec<(EpochTimestamp, f64)>,
    current_open_age_days_mean: f64,
    bors_retries: Vec<BorsRetry>,
}

#[derive(Clone, Debug, Queryable, Serialize)]
pub struct BorsRetry {
    repository: String,
    issue_num: i32,
    comment_id: i32,
    issue_title: String,
    merged: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReleaseSummary {
    nightlies: Vec<Release>,
    streak_summary: NightlyStreakSummary,
}

#[derive(Clone, Debug, Serialize, Eq, Ord, PartialEq, PartialOrd)]
pub struct BuildInfo {
    builder_name: String,
    os: String,
    env: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct NightlyStreakSummary {
    longest_length_days: u32,
    longest_start: NaiveDate,
    longest_end: NaiveDate,
    current_length_days: u32,
    last_failure: Option<NaiveDate>,
}

pub fn pr_summary(since: NaiveDate, until: NaiveDate) -> DashResult<PullRequestSummary> {
    let since = since.and_hms(0, 0, 0);
    let until = until.and_hms(23, 59, 59);

    let current_pr_age = try!(open_prs_avg_days_old());
    let prs_open_per_day = try!(prs_opened_per_day(since, until));
    let prs_close_per_day = try!(prs_closed_per_day(since, until));
    let prs_merge_per_day = try!(prs_merged_per_day(since, until));
    let pr_open_time = try!(prs_open_time_before_close(since, until));
    let bors_retries = try!(bors_retries_last_week());

    Ok(PullRequestSummary {
        opened_per_day: prs_open_per_day,
        closed_per_day: prs_close_per_day,
        merged_per_day: prs_merge_per_day,
        days_open_before_close: pr_open_time,
        current_open_age_days_mean: current_pr_age,
        bors_retries: bors_retries,
    })
}

pub fn nightly_summary(since: NaiveDate, until: NaiveDate) -> DashResult<ReleaseSummary> {
    let since = since.and_hms(0, 0, 0);
    let until = until.and_hms(23, 59, 59);

    Ok(ReleaseSummary {
        nightlies: nightly_releases(since, until)?,
        streak_summary: streaks()?,
    })
}

pub fn prs_opened_per_day(since: NaiveDateTime,
                          until: NaiveDateTime)
                          -> DashResult<Vec<(EpochTimestamp, i64)>> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("created_at::date as d, COUNT(*)"))
            .filter(created_at.ge(since))
            .filter(created_at.le(until))
            .group_by(d)
            .order(date(created_at).asc())
            .get_results::<(NaiveDate, i64)>(&*conn))
        .into_iter()
        .map(|(d, cnt)| (d.and_hms(12, 0, 0).timestamp(), cnt))
        .collect())
}

pub fn prs_closed_per_day(since: NaiveDateTime,
                          until: NaiveDateTime)
                          -> DashResult<Vec<(EpochTimestamp, i64)>> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("closed_at::date as d, COUNT(*)"))
            .filter(closed_at.is_not_null())
            .filter(closed_at.ge(since))
            .filter(closed_at.le(until))
            .group_by(&d)
            .order((&d).asc())
            .get_results::<(NaiveDate, i64)>(&*conn))
        .into_iter()
        .map(|(d, cnt)| (d.and_hms(12, 0, 0).timestamp(), cnt))
        .collect())
}

pub fn prs_merged_per_day(since: NaiveDateTime,
                          until: NaiveDateTime)
                          -> DashResult<Vec<(EpochTimestamp, i64)>> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(pullrequest.select(sql::<(Date, BigInt)>("merged_at::date as d, COUNT(*)"))
            .filter(merged_at.is_not_null())
            .filter(merged_at.ge(since))
            .filter(merged_at.le(until))
            .group_by(&d)
            .order((&d).asc())
            .get_results::<(NaiveDate, i64)>(&*conn))
        .into_iter()
        .map(|(d, cnt)| (d.and_hms(12, 0, 0).timestamp(), cnt))
        .collect())
}

pub fn prs_open_time_before_close(since: NaiveDateTime,
                                  until: NaiveDateTime)
                                  -> DashResult<Vec<(EpochTimestamp, f64)>> {
    use domain::schema::pullrequest::dsl::*;

    let conn = try!(DB_POOL.get());

    let w = sql::<Text>("iso_closed_week");
    let mut results = try!(pullrequest.select(sql::<(Double, Text)>("\
        AVG(EXTRACT(EPOCH FROM closed_at) - \
                                           EXTRACT(EPOCH FROM created_at)) / (60 * 60 * 24), \
                                           \
                                           EXTRACT(ISOYEAR FROM closed_at)::text || '-' || \
                                           EXTRACT(WEEK FROM closed_at)::text || '-6' AS \
                                           iso_closed_week"))
            .filter(closed_at.is_not_null())
            .filter(closed_at.ge(since))
            .filter(closed_at.le(until))
            .group_by(&w)
            .get_results::<(f64, String)>(&*conn))
        .into_iter()
        .map(|(time, week)| {
            let d = NaiveDate::parse_from_str(&week, "%G-%V-%w").unwrap();
            let d = d.and_hms(12, 0, 0).timestamp();
            (d, time)
        })
        .collect::<Vec<(EpochTimestamp, f64)>>();

    results.sort_by(|&(d1, _), &(d2, _)| d1.cmp(&d2));
    Ok(results)
}

pub fn open_prs_avg_days_old() -> DashResult<f64> {
    use domain::schema::pullrequest::dsl::*;
    let conn = try!(DB_POOL.get());
    Ok(try!(pullrequest.select(sql::<Double>("AVG(EXTRACT(EPOCH FROM (now() - created_at))) / \
                                              (60 * 60 * 24)"))
        .filter(closed_at.is_null())
        .first(&*conn)))
}

pub fn bors_retries_last_week() -> DashResult<Vec<BorsRetry>> {
    let conn = try!(DB_POOL.get());

    // waiting on associations to get this into proper typed queries

    Ok(try!(select(
        sql::<(VarChar, Integer, Integer, VarChar, Bool)>(
        "i.repository, i.number, ic.id, i.title, pr.merged_at IS NOT NULL \
        FROM issuecomment ic, issue i, pullrequest pr \
        WHERE \
        ic.body LIKE '%@bors%retry%' AND \
        i.id = ic.fk_issue AND \
        i.is_pull_request AND \
        ic.created_at > NOW() - '7 days'::interval AND \
        pr.repository = i.repository AND \
        pr.number = i.number \
        ORDER BY ic.created_at DESC"))
        .load(&*conn)))
}

pub fn nightly_releases(since: NaiveDateTime,
                        until: NaiveDateTime)
                        -> DashResult<Vec<Release>> {
    use domain::schema::release::dsl::*;

    let conn = try!(DB_POOL.get());

    let releases = try!(release.select((date, released))
        .filter(date.gt(since.date()))
        .filter(date.le(until.date()))
        .order(date.desc())
        .load::<Release>(&*conn));
    Ok(releases)
}

fn streaks() -> DashResult<NightlyStreakSummary> {
    use domain::schema::release::dsl::*;

    let conn = DB_POOL.get()?;

    let nightlies = release.select((date, released))
        .order(date.asc())
        .load::<Release>(&*conn)?;

    let mut last_failure = None;
    let mut longest_streak_length = 0;
    let mut longest_streak_start = nightlies[0].date;

    let mut streak_length = 0;
    let mut streak_start = nightlies[0].date;
    for nightly in nightlies {
        if nightly.released {
            if streak_length == 0 { // if first success
                streak_start = nightly.date
            }
            streak_length += 1;
        } else {
            last_failure = Some(nightly.date);
            if streak_length > longest_streak_length {
                longest_streak_length = streak_length;
                longest_streak_start = streak_start;
            }
            streak_length = 0;
        }
    }

    Ok(NightlyStreakSummary {
        longest_length_days: longest_streak_length,
        longest_start: longest_streak_start,
        longest_end: longest_streak_start + Duration::days(longest_streak_length as i64),
        current_length_days: streak_length,
        last_failure: last_failure,
    })
}

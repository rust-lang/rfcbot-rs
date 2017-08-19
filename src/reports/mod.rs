use chrono::{Duration, NaiveDate, NaiveDateTime};
use diesel::prelude::*;

use DB_POOL;
use domain::releases::Release;
use error::DashResult;

pub mod nag;

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

pub fn nightly_summary(since: NaiveDate, until: NaiveDate) -> DashResult<ReleaseSummary> {
    let since = since.and_hms(0, 0, 0);
    let until = until.and_hms(23, 59, 59);

    Ok(ReleaseSummary {
        nightlies: nightly_releases(since, until)?,
        streak_summary: streaks()?,
    })
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

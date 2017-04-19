use std::collections::BTreeMap;

use chrono::{Duration, NaiveDate, NaiveDateTime, UTC};
use diesel::expression::dsl::*;
use diesel::expression::AsExpression;
use diesel::prelude::*;
use diesel::select;
use diesel::types::{Array, BigInt, Bool, Date, Double, Integer, Nullable, Text, Timestamp, VarChar};

use DB_POOL;
use domain::buildbot::Build;
use domain::github::Issue;
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
pub struct IssueSummary {
    opened_per_day: Vec<(EpochTimestamp, i64)>,
    closed_per_day: Vec<(EpochTimestamp, i64)>,
    days_open_before_close: Vec<(EpochTimestamp, f64)>,
    current_open_age_days_mean: f64,
    num_open_p_high_issues: i64,
    num_open_regression_nightly_issues: i64,
    num_open_regression_beta_issues: i64,
    num_open_regression_stable_issues: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReleaseSummary {
    nightlies: Vec<Release>,
    builder_times_mins: Vec<(String, Vec<(EpochTimestamp, f64)>)>,
    streak_summary: NightlyStreakSummary,
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildbotSummary {
    per_builder_times_mins: Vec<(String, Vec<(EpochTimestamp, f64)>)>,
    per_builder_failures: Vec<(String, Vec<(EpochTimestamp, i64)>)>,
    failures_last_day: Vec<Build>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NightlyStreakSummary {
    longest_length_days: u32,
    longest_start: NaiveDate,
    longest_end: NaiveDate,
    current_length_days: u32,
    last_failure: Option<NaiveDate>,
}

#[derive(Clone, Debug, Serialize)]
pub struct HotIssueSummary {
    issues: Vec<Issue>,
}

pub fn issue_summary(since: NaiveDate, until: NaiveDate) -> DashResult<IssueSummary> {
    let since = since.and_hms(0, 0, 0);
    let until = until.and_hms(23, 59, 59);

    let current_issue_age = try!(open_issues_avg_days_old());
    let issue_open_time = try!(issues_open_time_before_close(since, until));
    let issues_open_per_day = try!(issues_opened_per_day(since, until));
    let issues_close_per_day = try!(issues_closed_per_day(since, until));
    let num_p_high = try!(open_issues_with_label("P-high"));
    let nightly_regress = try!(open_issues_with_label("regression-from-stable-to-nightly"));
    let beta_regress = try!(open_issues_with_label("regression-from-stable-to-beta"));
    let stable_regress = try!(open_issues_with_label("regression-from-stable-to-stable"));

    Ok(IssueSummary {
        opened_per_day: issues_open_per_day,
        closed_per_day: issues_close_per_day,
        days_open_before_close: issue_open_time,
        current_open_age_days_mean: current_issue_age,
        num_open_p_high_issues: num_p_high,
        num_open_regression_nightly_issues: nightly_regress,
        num_open_regression_beta_issues: beta_regress,
        num_open_regression_stable_issues: stable_regress,
    })
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

pub fn ci_summary(since: NaiveDate, until: NaiveDate) -> DashResult<BuildbotSummary> {
    let since = since.and_hms(0, 0, 0);
    let until = until.and_hms(23, 59, 59);

    let per_builder_times = try!(buildbot_build_times(since, until, "auto-%"));
    let per_builder_fails = try!(buildbot_failures_by_day(since, until));
    let failures_last_day = try!(buildbot_failures_last_24_hours());

    Ok(BuildbotSummary {
        per_builder_times_mins: per_builder_times,
        per_builder_failures: per_builder_fails,
        failures_last_day: failures_last_day,
    })
}

pub fn nightly_summary(since: NaiveDate, until: NaiveDate) -> DashResult<ReleaseSummary> {
    let since = since.and_hms(0, 0, 0);
    let until = until.and_hms(23, 59, 59);

    let nightlies = try!(nightly_releases(since, until));
    let build_times = try!(buildbot_build_times(since, until, "nightly-%"));
    let streaks = try!(streaks());

    Ok(ReleaseSummary {
        nightlies: nightlies,
        builder_times_mins: build_times,
        streak_summary: streaks,
    })
}

pub fn hot_issues_summary() -> DashResult<HotIssueSummary> {
    Ok(HotIssueSummary {issues: hottest_issues_last_month()?})
}

pub fn hottest_issues_last_month() -> DashResult<Vec<Issue>> {

    let conn = try!(DB_POOL.get());

    Ok(try!(select(sql::<(Integer,
                          Integer,
                          Nullable<Integer>,
                          Integer,
                          Nullable<Integer>,
                          Bool,
                          Bool,
                          VarChar,
                          VarChar,
                          Bool,
                          Nullable<Timestamp>,
                          Timestamp,
                          Timestamp,
                          Array<VarChar>,
                          VarChar)>("i.id, \
          i.number, \
          i.fk_milestone, \
          i.fk_user, \
          i.fk_assignee, \
          i.open, \
          i.is_pull_request, \
          i.title, \
          i.body, \
          i.locked, \
          i.closed_at, \
          i.created_at, \
          i.updated_at, \
          i.labels, \
          i.repository \
        FROM issue i, issuecomment ic, githubuser u \
        WHERE \
          i.id = ic.fk_issue AND \
          ic.created_at >= NOW() - '14 days'::interval AND \
          i.open AND \
          ic.fk_user = u.id AND \
          u.login != 'bors' AND \
          ic.body NOT LIKE '%@bors%' \
        GROUP BY \
          i.id, \
          i.number, \
          i.fk_milestone, \
          i.fk_user, \
          i.fk_assignee, \
          i.open, \
          i.is_pull_request, \
          i.title, \
          i.body, \
          i.locked, \
          i.closed_at, \
          i.created_at, \
          i.updated_at, \
          i.labels, \
          i.repository \
        ORDER BY COUNT(ic.*) DESC \
        LIMIT 50"))
        .load::<Issue>(&*conn)))
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

pub fn issues_opened_per_day(since: NaiveDateTime,
                             until: NaiveDateTime)
                             -> DashResult<Vec<(EpochTimestamp, i64)>> {
    use domain::schema::issue::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(issue.select(sql::<(Date, BigInt)>("created_at::date as d, COUNT(*)"))
            .filter(created_at.ge(since))
            .filter(created_at.le(until))
            .group_by(&d)
            .order((&d).asc())
            .get_results::<(NaiveDate, i64)>(&*conn))
        .into_iter()
        .map(|(t, c)| (t.and_hms(12, 0, 0).timestamp(), c))
        .collect())
}

pub fn issues_closed_per_day(since: NaiveDateTime,
                             until: NaiveDateTime)
                             -> DashResult<Vec<(EpochTimestamp, i64)>> {
    use domain::schema::issue::dsl::*;

    let conn = try!(DB_POOL.get());
    let d = sql::<Date>("d");
    Ok(try!(issue.select(sql::<(Date, BigInt)>("closed_at::date as d, COUNT(*)"))
            .filter(closed_at.is_not_null())
            .filter(closed_at.ge(since))
            .filter(closed_at.le(until))
            .group_by(&d)
            .order((&d).asc())
            .get_results::<(NaiveDate, i64)>(&*conn))
        .into_iter()
        .map(|(t, c)| (t.and_hms(12, 0, 0).timestamp(), c))
        .collect())
}

pub fn issues_open_time_before_close(since: NaiveDateTime,
                                     until: NaiveDateTime)
                                     -> DashResult<Vec<(EpochTimestamp, f64)>> {
    use domain::schema::issue::dsl::*;

    let conn = try!(DB_POOL.get());

    let w = sql::<Text>("iso_closed_week");
    let mut results = try!(issue.select(sql::<(Double, Text)>("\
                                             \
                                           AVG(EXTRACT(EPOCH FROM closed_at) - EXTRACT(EPOCH \
                                           FROM created_at)) / (60 * 60 * 24), \
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

pub fn open_issues_avg_days_old() -> DashResult<f64> {
    use domain::schema::issue::dsl::*;
    let conn = try!(DB_POOL.get());
    Ok(try!(issue.select(sql::<Double>("AVG(EXTRACT(EPOCH FROM (now() - created_at))) / \
                                              (60 * 60 * 24)"))
        .filter(closed_at.is_null())
        .first(&*conn)))
}

pub fn open_issues_with_label(label: &str) -> DashResult<i64> {
    use domain::schema::issue::dsl::*;
    let conn = try!(DB_POOL.get());

    Ok(try!(issue.select(count_star())
        .filter(closed_at.is_null())
        .filter(AsExpression::<Text>::as_expression(label).eq(any(labels)))
        .first(&*conn)))
}

pub fn buildbot_build_times(since: NaiveDateTime,
                            until: NaiveDateTime,
                            builder_pattern: &str)
                            -> DashResult<Vec<(String, Vec<(EpochTimestamp, f64)>)>> {
    use domain::schema::build::dsl::*;

    let conn = try!(DB_POOL.get());

    let name_date = sql::<(Text, Date)>("builder_name, date(start_time)");

    let triples = try!(build.select((&name_date, sql::<Double>("(AVG(duration_secs) / 60)::float")))
        .filter(successful)
        .filter(start_time.is_not_null())
        .filter(start_time.ge(since))
        .filter(start_time.le(until))
        .filter(builder_name.like(builder_pattern))
        .group_by(&name_date)
        .order((&name_date).asc())
        .load::<((String, NaiveDate), f64)>(&*conn));

    let mut results = BTreeMap::new();
    for ((builder, date), build_minutes) in triples {
        results.entry(builder)
            .or_insert_with(Vec::new)
            .push((date.and_hms(12, 0, 0).timestamp(), build_minutes));
    }

    Ok(results.into_iter().collect())
}

pub fn buildbot_failures_by_day(since: NaiveDateTime,
                                until: NaiveDateTime)
                                -> DashResult<Vec<(String, Vec<(EpochTimestamp, i64)>)>> {
    use domain::schema::build::dsl::*;

    let conn = try!(DB_POOL.get());

    let name_date = sql::<(Text, Date)>("builder_name, date(start_time)");

    let triples = try!(build.select((&name_date, sql::<BigInt>("COUNT(*)")))
        .filter(successful.ne(true))
        .filter(start_time.is_not_null())
        .filter(start_time.ge(since))
        .filter(start_time.le(until))
        .filter(builder_name.like("auto-%"))
        .filter(message.not_like("%xception%nterrupted%"))
        .group_by(&name_date)
        .order((&name_date).asc())
        .load::<((String, NaiveDate), i64)>(&*conn));

    let mut results = BTreeMap::new();
    for ((builder, date), build_minutes) in triples {
        results.entry(builder)
            .or_insert_with(Vec::new)
            .push((date.and_hms(12, 0, 0).timestamp(), build_minutes));
    }

    Ok(results.into_iter().collect())
}

pub fn buildbot_failures_last_24_hours() -> DashResult<Vec<Build>> {
    use domain::schema::build::dsl::*;

    let conn = try!(DB_POOL.get());

    let one_day_ago = UTC::now().naive_utc() - Duration::days(1);

    Ok(try!(build.select((number,
                 builder_name,
                 successful,
                 message,
                 duration_secs,
                 start_time,
                 end_time))
        .filter(successful.ne(true))
        .filter(end_time.is_not_null())
        .filter(end_time.ge(one_day_ago))
        .filter(message.not_like("%xception%nterrupted%"))
        .order(end_time.desc())
        .load::<Build>(&*conn)))
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

    let nightlies = release.select((date, released)).load::<Release>(&*conn)?;

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

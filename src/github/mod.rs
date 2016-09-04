// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).


pub mod client;
pub mod models;
mod nag;

use std::collections::BTreeSet;
use std::cmp;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, UTC};
use diesel::expression::dsl::*;
use diesel::prelude::*;
use diesel;

use DB_POOL;
use domain::github::*;
use domain::schema::*;
use error::DashResult;

use self::client::Client;
use self::models::PullRequestFromJson;

lazy_static! {
    pub static ref GH: Client = Client::new();
}

pub fn most_recent_update(repo: &str) -> DashResult<DateTime<UTC>> {
    info!("finding most recent github updates");

    let default_date = NaiveDateTime::new(NaiveDate::from_ymd(2015, 5, 15),
                                          NaiveTime::from_hms(0, 0, 0));

    let conn = try!(DB_POOL.get());

    let issues_updated: NaiveDateTime = {
        use domain::schema::issue::dsl::*;
        match issue.select(max(updated_at)).filter(repository.eq(repo)).first(&*conn) {
            Ok(dt) => dt,
            Err(_) => default_date,
        }
    };

    let prs_updated: NaiveDateTime = {
        use domain::schema::pullrequest::dsl::*;
        match pullrequest.select(max(updated_at)).filter(repository.eq(repo)).first(&*conn) {
            Ok(dt) => dt,
            Err(_) => default_date,
        }
    };

    let comments_updated: NaiveDateTime = {
        use domain::schema::issuecomment::dsl::*;
        match issuecomment.select(max(updated_at)).filter(repository.eq(repo)).first(&*conn) {
            Ok(dt) => dt,
            Err(_) => default_date,
        }
    };

    let most_recent = cmp::min(issues_updated, cmp::min(prs_updated, comments_updated));

    Ok(DateTime::from_utc(most_recent, UTC))
}

pub fn ingest_since(repo: &str, start: DateTime<UTC>) -> DashResult<()> {
    info!("fetching all {} issues and comments since {}", repo, start);
    let issues = try!(GH.issues_since(repo, start));
    let comments = try!(GH.comments_since(repo, start));

    let mut prs: Vec<PullRequestFromJson> = vec![];
    for issue in &issues {
        // sleep(Duration::from_millis(github::client::DELAY));
        if let Some(ref pr_info) = issue.pull_request {
            match GH.fetch_pull_request(pr_info) {
                Ok(pr) => prs.push(pr),
                Err(why) => {
                    error!("ERROR fetching PR info: {:?}", why);
                    break;
                }
            }
        }
    }

    debug!("num pull requests updated since {}: {:#?}",
           &start,
           prs.len());

    debug!("num issues updated since {}: {:?}", &start, issues.len());
    debug!("num comments updated since {}: {:?}",
           &start,
           comments.len());

    debug!("let's insert some stuff in the database");

    let conn = try!(DB_POOL.get());

    // make sure we have all of the users to ensure referential integrity
    let mut users = BTreeSet::new();
    for issue in &issues {
        users.insert(issue.user.clone());

        if issue.assignee.is_some() {
            users.insert(issue.assignee.clone().unwrap());
        }

        if issue.milestone.is_some() {
            users.insert(issue.milestone.clone().unwrap().creator);
        }
    }

    for comment in &comments {
        users.insert(comment.user.clone());
    }

    for pr in &prs {
        if pr.assignee.is_some() {
            users.insert(pr.assignee.clone().unwrap());
        }
    }

    // make sure all the users are present in the database
    for user in users {
        let exists = githubuser::table.find(user.id).get_result::<GitHubUser>(&*conn).is_ok();

        if exists {
            try!(diesel::update(githubuser::table.find(user.id)).set(&user).execute(&*conn));
        } else {
            try!(diesel::insert(&user).into(githubuser::table).execute(&*conn));
        }
    }

    // insert the issues, milestones, and labels
    for issue in issues {
        let (i, milestone) = issue.with_repo(repo);

        if let Some(milestone) = milestone {
            let exists = milestone::table.find(milestone.id)
                .get_result::<Milestone>(&*conn)
                .is_ok();
            if exists {
                try!(diesel::update(milestone::table.find(milestone.id))
                    .set(&milestone)
                    .execute(&*conn));
            } else {
                try!(diesel::insert(&milestone).into(milestone::table).execute(&*conn));
            }
        }

        {
            use domain::schema::issue::dsl::*;

            let exists = issue.select(id)
                .filter(number.eq(&i.number))
                .filter(repository.eq(&i.repository))
                .first::<i32>(&*conn)
                .ok();

            if let Some(current_id) = exists {
                try!(diesel::update(issue.find(current_id)).set(&i.complete(current_id)).execute(&*conn));
            } else {
                try!(diesel::insert(&i).into(issue).execute(&*conn));
            }
        }
    }

    let mut domain_comments = Vec::new();

    // insert the comments
    for comment in comments {
        let comment: IssueComment = try!(comment.with_repo(repo));

        if issuecomment::table.find(comment.id).get_result::<IssueComment>(&*conn).is_ok() {
            try!(diesel::update(issuecomment::table.find(comment.id))
                .set(&comment)
                .execute(&*conn));
        } else {
            try!(diesel::insert(&comment).into(issuecomment::table).execute(&*conn));

            // we don't want to double-process comments
            domain_comments.push(comment);
        }
    }

    for pr in prs {
        use domain::schema::pullrequest::dsl::*;

        let pr: PullRequest = pr.with_repo(repo);

        let existing_id = pullrequest.select(id)
            .filter(number.eq(&pr.number))
            .filter(repository.eq(&pr.repository))
            .first::<i32>(&*conn)
            .ok();

        if let Some(current_id) = existing_id {
            try!(diesel::update(pullrequest.find(current_id)).set(&pr).execute(&*conn));
        } else {
            try!(diesel::insert(&pr).into(pullrequest).execute(&*conn));
        }
    }

    // now that all updates have been registered, update any applicable nags
    match nag::update_nags(domain_comments) {
        Ok(()) => Ok(()),
        Err(why) => {
            error!("Problem updating FCPs: {:?}", &why);
            Err(why)
        }
    }
}

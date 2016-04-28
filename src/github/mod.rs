// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).


pub mod client;
pub mod models;

use std::collections::BTreeSet;
use std::cmp;

use chrono::{DateTime, NaiveDateTime, UTC};
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
    static ref GH: Client = Client::new();
}

pub fn most_recent_update() -> DashResult<DateTime<UTC>> {
    info!("finding most recent github updates");

    let conn = try!(DB_POOL.get());

    let issues_updated: NaiveDateTime = {
        use domain::schema::issue::dsl::*;
        try!(issue.select(max(updated_at)).first(&*conn))
    };

    let prs_updated: NaiveDateTime = {
        use domain::schema::pullrequest::dsl::*;
        try!(pullrequest.select(max(updated_at)).first(&*conn))
    };

    let comments_updated: NaiveDateTime = {
        use domain::schema::issuecomment::dsl::*;
        try!(issuecomment.select(max(updated_at)).first(&*conn))
    };

    let most_recent = cmp::min(issues_updated, cmp::min(prs_updated, comments_updated));

    Ok(DateTime::from_utc(most_recent, UTC))
}

pub fn ingest_since(start: DateTime<UTC>) -> DashResult<()> {
    info!("fetching all rust-lang/rust issues and comments since {}",
          start);
    let issues = try!(GH.issues_since(start));
    let comments = try!(GH.comments_since(start));

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
        let (issue, milestone) = issue.into();

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

        let exists = issue::table.find(issue.number).get_result::<Issue>(&*conn).is_ok();
        if exists {
            try!(diesel::update(issue::table.find(issue.number)).set(&issue).execute(&*conn));
        } else {
            try!(diesel::insert(&issue).into(issue::table).execute(&*conn));
        }
    }

    // insert the comments
    for comment in comments {
        let comment: IssueComment = comment.into();

        if issuecomment::table.find(comment.id).get_result::<IssueComment>(&*conn).is_ok() {
            try!(diesel::update(issuecomment::table.find(comment.id))
                     .set(&comment)
                     .execute(&*conn));
        } else {
            try!(diesel::insert(&comment).into(issuecomment::table).execute(&*conn));
        }
    }

    for pr in prs {
        let pr: PullRequest = pr.into();

        let exists = pullrequest::table.find(pr.number).get_result::<PullRequest>(&*conn).is_ok();

        if exists {
            try!(diesel::update(pullrequest::table.find(pr.number)).set(&pr).execute(&*conn));
        } else {
            try!(diesel::insert(&pr).into(pullrequest::table).execute(&*conn));
        }
    }

    Ok(())
}

// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).


pub mod client;
pub mod models;
mod nag;
pub mod webhooks;

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
use self::models::{CommentFromJson, IssueFromJson, PullRequestFromJson};

lazy_static! {
    pub static ref GH: Client = Client::new();
}

pub fn most_recent_update(repo: &str) -> DashResult<DateTime<UTC>> {
    info!("finding most recent github updates");

    let default_date = NaiveDateTime::new(NaiveDate::from_ymd(2015, 5, 15),
                                          NaiveTime::from_hms(0, 0, 0));

    let conn = &*DB_POOL.get()?;

    let updated: NaiveDateTime = {
        use domain::schema::githubsync::dsl::*;
        githubsync.select(ran_at)
            .filter(successful.eq(true))
            .order(ran_at.desc())
            .first(conn)
            .unwrap_or(default_date)
    };

    Ok(DateTime::from_utc(updated, UTC))
}

pub fn ingest_since(repo: &str, start: DateTime<UTC>) -> DashResult<()> {
    let ingest_start = UTC::now().naive_utc();

    info!("fetching all {} issues and comments since {}", repo, start);
    let issues = try!(GH.issues_since(repo, start));
    let mut comments = try!(GH.comments_since(repo, start));
    // make sure we process the new comments in creation order
    comments.sort_by_key(|c| c.created_at);

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

    // make sure we have all of the users to ensure referential integrity
    for issue in issues {
        let issue_number = issue.number;
        match handle_issue(issue, repo) {
            Ok(()) => (),
            Err(why) => {
                error!("Error processing issue {}#{}: {:?}",
                       repo,
                       issue_number,
                       why)
            }
        }
    }

    // insert the comments
    for comment in comments {
        let comment_id = comment.id;
        match handle_comment(comment, repo) {
            Ok(()) => (),
            Err(why) => {
                error!("Error processing comment {}#{}: {:?}",
                       repo,
                       comment_id,
                       why)
            }
        }
    }

    for pr in prs {
        let pr_number = pr.number;
        match handle_pr(pr, repo) {
            Ok(()) => (),
            Err(why) => error!("Error processing PR {}#{}: {:?}", repo, pr_number, why),
        }
    }

    {
        let conn = &*DB_POOL.get()?;
        // insert a successful sync record
        use domain::schema::githubsync::dsl::*;
        let sync_record = GitHubSyncPartial {
            successful: true,
            ran_at: ingest_start,
            message: None,
        };

        diesel::insert(&sync_record).into(githubsync).execute(conn)?;
    }

    Ok(())
}

pub fn handle_pr(pr: PullRequestFromJson, repo: &str) -> DashResult<()> {
    use domain::schema::pullrequest::dsl::*;

    let conn = DB_POOL.get()?;

    if let Some(ref assignee) = pr.assignee {
        handle_user(assignee)?;
    }

    let pr: PullRequest = pr.with_repo(repo);

    let existing_id = pullrequest.select(id)
        .filter(number.eq(&pr.number))
        .filter(repository.eq(&pr.repository))
        .first::<i32>(&*conn)
        .ok();

    if let Some(current_id) = existing_id {
        diesel::update(pullrequest.find(current_id)).set(&pr).execute(&*conn)?;
    } else {
        diesel::insert(&pr).into(pullrequest).execute(&*conn)?;
    }

    Ok(())
}

pub fn handle_comment(comment: CommentFromJson, repo: &str) -> DashResult<()> {
    handle_user(&comment.user)?;

    let conn = DB_POOL.get()?;

    let comment: IssueComment = comment.with_repo(repo)?;

    if issuecomment::table.find(comment.id).get_result::<IssueComment>(&*conn).is_ok() {
        diesel::update(issuecomment::table.find(comment.id)).set(&comment)
            .execute(&*conn)?;
        Ok(())
    } else {
        diesel::insert(&comment).into(issuecomment::table).execute(&*conn)?;

        // we don't want to double-process comments
        // now that all updates have been registered, update any applicable nags
        match nag::update_nags(&comment) {
            Ok(()) => Ok(()),
            Err(why) => {
                error!("Problem updating FCPs: {:?}", &why);
                Err(why)
            }
        }
    }
}

pub fn handle_issue(issue: IssueFromJson, repo: &str) -> DashResult<()> {
    let conn = DB_POOL.get()?;

    // user handling
    handle_user(&issue.user)?;
    if let Some(ref assignee) = issue.assignee {
        handle_user(assignee)?;
    }
    if let Some(ref milestone) = issue.milestone {
        handle_user(&milestone.creator)?;
    }

    let (i, milestone) = issue.with_repo(repo);

    // handle milestones
    if let Some(milestone) = milestone {
        let exists = milestone::table.find(milestone.id)
            .get_result::<Milestone>(&*conn)
            .is_ok();
        if exists {
            diesel::update(milestone::table.find(milestone.id)).set(&milestone).execute(&*conn)?;
        } else {
            diesel::insert(&milestone).into(milestone::table).execute(&*conn)?;
        }
    }

    // handle issue itself
    {
        use domain::schema::issue::dsl::*;

        let exists = issue.select(id)
            .filter(number.eq(&i.number))
            .filter(repository.eq(&i.repository))
            .first::<i32>(&*conn)
            .ok();

        if let Some(current_id) = exists {
            diesel::update(issue.find(current_id)).set(&i.complete(current_id))
                .execute(&*conn)?;
        } else {
            diesel::insert(&i).into(issue).execute(&*conn)?;
        }
    }

    Ok(())
}

pub fn handle_user(user: &GitHubUser) -> DashResult<()> {
    let conn = DB_POOL.get()?;
    let exists = githubuser::table.find(user.id).get_result::<GitHubUser>(&*conn).is_ok();

    if exists {
        diesel::update(githubuser::table.find(user.id)).set(user).execute(&*conn)?;
    } else {
        diesel::insert(user).into(githubuser::table).execute(&*conn)?;
    }

    Ok(())
}

// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).


pub mod client;
pub mod models;
mod command;
mod nag;
pub mod webhooks;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::pg::upsert::*;
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

pub fn most_recent_update() -> DashResult<DateTime<Utc>> {
    info!("finding most recent github updates");

    let default_date = NaiveDateTime::new(NaiveDate::from_ymd(2015, 5, 15),
                                          NaiveTime::from_hms(0, 0, 0));

    let conn = &*DB_POOL.get()?;

    let updated: NaiveDateTime = {
        use domain::schema::githubsync::dsl::*;
        githubsync
            .select(ran_at)
            .filter(successful.eq(true))
            .order(ran_at.desc())
            .first(conn)
            .unwrap_or(default_date)
    };

    Ok(DateTime::from_utc(updated, Utc))
}

pub fn record_successful_update(ingest_start: NaiveDateTime) -> DashResult<()> {
    let conn = &*DB_POOL.get()?;
    // insert a successful sync record
    use domain::schema::githubsync::dsl::*;
    let sync_record = GitHubSyncPartial {
        successful: true,
        ran_at: ingest_start,
        message: None,
    };

    diesel::insert(&sync_record).into(githubsync).execute(conn)?;
    Ok(())
}

pub fn ingest_since(repo: &str, start: DateTime<Utc>) -> DashResult<()> {
    info!("fetching all {} issues and comments since {}", repo, start);
    let issues = GH.issues_since(repo, start)?;
    let mut comments = GH.comments_since(repo, start)?;
    // make sure we process the new comments in creation order
    comments.sort_by_key(|c| c.created_at);

    let mut prs: Vec<PullRequestFromJson> = vec![];
    for issue in &issues {
        // sleep(Duration::from_millis(github::client::DELAY));
        if let Some(ref pr_info) = issue.pull_request {
            prs.push(ok_or!(GH.fetch_pull_request(pr_info), why => {
                error!("ERROR fetching PR info: {:?}", why);
                break;
            }));
        }
    }

    debug!("num pull requests updated since {}: {:#?}",
           &start,
           prs.len());

    debug!("num issues updated since {}: {:?}", &start, issues.len());
    debug!("num comments updated since {}: {:?}",
           &start,
           comments.len());

    let conn = &*DB_POOL.get()?;
    debug!("let's insert some stuff in the database");


    // make sure we have all of the users to ensure referential integrity
    for issue in issues {
        let issue_number = issue.number;
        ok_or!(handle_issue(conn, issue, repo), why =>
            error!("Error processing issue {}#{}: {:?}",
                   repo, issue_number, why));
    }

    // insert the comments
    for comment in comments {
        let comment_id = comment.id;
        ok_or!(handle_comment(conn, comment, repo), why =>
            error!("Error processing comment {}#{}: {:?}",
                   repo, comment_id, why));
    }

    for pr in prs {
        let pr_number = pr.number;
        ok_or!(handle_pr(conn, pr, repo), why =>
            error!("Error processing PR {}#{}: {:?}", repo, pr_number, why));
    }

    Ok(())
}

pub fn handle_pr(conn: &PgConnection, pr: PullRequestFromJson, repo: &str) -> DashResult<()> {
    use domain::schema::pullrequest::dsl::*;
    if let Some(ref assignee) = pr.assignee {
        handle_user(conn, assignee)?;
    }

    let pr: PullRequest = pr.with_repo(repo);
    diesel::insert(&pr.on_conflict((repository, number), do_update().set(&pr)))
        .into(pullrequest)
        .execute(conn)?;
    Ok(())
}

pub fn handle_comment(conn: &PgConnection, comment: CommentFromJson, repo: &str) -> DashResult<()> {
    handle_user(conn, &comment.user)?;

    let comment: IssueComment = comment.with_repo(repo)?;

    // We only want to run `nag::update_nags` on insert to avoid
    // double-processing commits, so we can't use upsert here
    if issuecomment::table
           .find(comment.id)
           .get_result::<IssueComment>(conn)
           .is_ok() {
        diesel::update(issuecomment::table.find(comment.id))
            .set(&comment)
            .execute(conn)?;
    } else {
        diesel::insert(&comment)
            .into(issuecomment::table)
            .execute(conn)?;

        ok_or!(nag::update_nags(&comment), why => {
            error!("Problem updating FCPs: {:?}", &why);
            throw!(why);
        });
    }

    Ok(())
}

pub fn handle_issue(conn: &PgConnection, issue: IssueFromJson, repo: &str) -> DashResult<()> {
    // user handling
    handle_user(conn, &issue.user)?;
    if let Some(ref assignee) = issue.assignee {
        handle_user(conn, assignee)?;
    }
    if let Some(ref milestone) = issue.milestone {
        handle_user(conn, &milestone.creator)?;
    }

    let (i, milestone) = issue.with_repo(repo);

    if let Some(milestone) = milestone {
        diesel::insert(&milestone.on_conflict(milestone::id, do_update().set(&milestone)))
            .into(milestone::table)
            .execute(conn)?;
    }

    // handle issue itself
    {
        use domain::schema::issue::dsl::*;
        diesel::insert(&i.on_conflict((repository, number), do_update().set(&i)))
            .into(issue)
            .execute(conn)?;
    }

    Ok(())
}

pub fn handle_user(conn: &PgConnection, user: &GitHubUser) -> DashResult<()> {
    diesel::insert(&user.on_conflict(githubuser::id, do_update().set(user)))
        .into(githubuser::table)
        .execute(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_handle_user() {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let conn = PgConnection::establish(&db_url)
            .expect(&format!("Error connecting to {}", db_url));

        let user = GitHubUser {
            id: -1,
            login: "A".to_string(),
        };
        let query = githubuser::table.filter(githubuser::id.eq(user.id));

        // User should not exist
        assert_eq!(query.load::<GitHubUser>(&conn), Ok(vec![]));

        // User has been inserted
        handle_user(&conn, &user).expect("Unable to handle user!");
        assert_eq!(query.load::<GitHubUser>(&conn), Ok(vec![user.clone()]));

        // User has been inserted, but nothing changed
        handle_user(&conn, &user).expect("Unable to handle user!");
        assert_eq!(query.load::<GitHubUser>(&conn), Ok(vec![user.clone()]));

        // User has been inserted, but login has changed
        let new_user = GitHubUser {
            id: user.id,
            login: user.login + "_new",
        };
        handle_user(&conn, &new_user).expect("Unable to handle user!");
        assert_eq!(query.load::<GitHubUser>(&conn), Ok(vec![new_user.clone()]));

        // Clean up after ourselves
        diesel::delete(githubuser::table.filter(githubuser::id.eq(user.id)))
            .execute(&conn)
            .expect("Failed to clear database");
    }
}

// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, FilterDsl, LoadDsl, Queryable, SaveChangesDsl, Table};

use super::schema::*;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[insertable_into(githubuser)]
#[changeset_for(githubuser)]
pub struct GitHubUser {
    pub id: i32,
    pub login: String,
}

#[derive(Debug, Queryable)]
pub struct IssueLabel {
    pub fk_issue: i32,
    pub label: String,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[insertable_into(milestone)]
#[changeset_for(milestone, treat_none_as_null="true")]
pub struct Milestone {
    pub id: i32,
    pub number: i32,
    pub open: bool,
    pub title: String,
    pub description: Option<String>,
    pub fk_creator: i32,
    pub open_issues: i32,
    pub closed_issues: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
    pub due_on: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[insertable_into(issue)]
#[changeset_for(issue, treat_none_as_null="true")]
pub struct Issue {
    pub number: i32,
    pub fk_milestone: Option<i32>,
    pub fk_user: i32,
    pub fk_assignee: Option<i32>,
    pub open: bool,
    pub is_pull_request: bool,
    pub title: String,
    pub body: String,
    pub locked: bool,
    pub closed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[insertable_into(issuecomment)]
#[changeset_for(issuecomment, treat_none_as_null="true")]
pub struct IssueComment {
    pub id: i32,
    pub fk_issue: i32,
    pub fk_user: i32,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[insertable_into(pullrequest)]
#[changeset_for(pullrequest, treat_none_as_null="true")]
pub struct PullRequest {
    pub number: i32,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub fk_assignee: Option<i32>,
    pub fk_milestone: Option<i32>,
    pub locked: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
    pub merged_at: Option<NaiveDateTime>,
    pub commits: i32,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
}

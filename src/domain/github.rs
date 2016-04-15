use std::u32;

use chrono::NaiveDateTime;

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: u32,
    pub login: String,
}

#[derive(Debug, Queryable)]
pub struct IssueLabel {
    pub fk_issue: u32,
    pub label: String,
    pub color: String,
}

#[derive(Debug, Queryable)]
pub struct Milestone {
    pub id: u32,
    pub number: u32,
    pub open: bool,
    pub title: String,
    pub description: Option<String>,
    pub creator: User,
    pub open_issues: u32,
    pub closed_issues: u32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
    pub due_on: Option<NaiveDateTime>,
}

#[derive(Debug, Queryable)]
pub struct Issue {
    pub number: u32,
    pub fk_milestone: Option<u32>,
    pub fk_user: u32,
    pub fk_assignee: Option<u32>,
    pub open: bool,
    pub title: String,
    pub body: String,
    pub locked: bool,
    pub closed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Queryable)]
pub struct IssueComment {
    pub id: u32,
    pub fk_issue: u32,
    pub fk_user: u32,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std::collections::BTreeMap;
use std::i32;

use chrono::{DateTime, Utc};

use DB_POOL;
use domain::github::{IssueComment, IssuePartial, Milestone, PullRequest, GitHubUser};
use error::DashResult;

#[derive(Clone, Debug, Deserialize)]
pub struct MilestoneFromJson {
    pub id: i32,
    pub number: i32,
    pub state: String,
    pub title: String,
    pub description: Option<String>,
    pub creator: GitHubUser,
    pub open_issues: i32,
    pub closed_issues: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub due_on: Option<DateTime<Utc>>,
}

impl MilestoneFromJson {
    pub fn with_repo(self, repo: &str) -> Milestone {
        Milestone {
            id: self.id,
            number: self.number,
            open: self.state == "open",
            title: self.title.replace(0x00 as char, ""),
            description: self.description.map(|s| s.replace(0x00 as char, "")),
            fk_creator: self.creator.id,
            open_issues: self.open_issues,
            closed_issues: self.closed_issues,
            created_at: self.created_at.naive_utc(),
            updated_at: self.updated_at.naive_utc(),
            closed_at: self.closed_at.map(|t| t.naive_utc()),
            due_on: self.due_on.map(|t| t.naive_utc()),
            repository: repo.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LabelFromJson {
    name: String,
    color: String,
}

pub type PullRequestUrls = BTreeMap<String, String>;

#[derive(Debug, Deserialize)]
pub struct IssueFromJson {
    pub number: i32,
    pub user: GitHubUser,
    pub assignee: Option<GitHubUser>,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub labels: Option<Vec<LabelFromJson>>,
    pub milestone: Option<MilestoneFromJson>,
    pub locked: bool,
    pub comments: i32,
    pub pull_request: Option<PullRequestUrls>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub comments_url: String,
}

impl IssueFromJson {
    pub fn with_repo(self, repo: &str) -> (IssuePartial, Option<Milestone>) {
        let issue = IssuePartial {
            number: self.number,
            fk_milestone: self.milestone.as_ref().map(|m| m.id),
            fk_user: self.user.id,
            fk_assignee: self.assignee.map(|a| a.id),
            open: self.state == "open",
            is_pull_request: self.pull_request.is_some(),
            title: self.title.replace(0x00 as char, ""),
            body: self.body
                .unwrap_or_else(String::new)
                .replace(0x00 as char, ""),
            locked: self.locked,
            closed_at: self.closed_at.map(|t| t.naive_utc()),
            created_at: self.created_at.naive_utc(),
            updated_at: self.updated_at.naive_utc(),
            labels: match self.labels {
                Some(json_labels) => json_labels.into_iter().map(|l| l.name).collect(),
                None => vec![],
            },
            repository: repo.to_string(),
        };

        (issue, self.milestone.map(|m| m.with_repo(repo)))
    }
}

#[derive(Debug, Deserialize)]
pub struct CommentFromJson {
    pub id: i32,
    pub html_url: String,
    pub body: String,
    pub user: GitHubUser,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CommentFromJson {
    pub fn with_repo(self, repo: &str) -> DashResult<IssueComment> {
        use diesel::prelude::*;
        use domain::schema::issue::dsl::*;

        let issue_number = self.html_url
            .split('#')
            .next()
            .map(|r| r.split('/').last().map(|n| n.parse::<i32>()));

        let issue_number = match issue_number {
            Some(Some(Ok(n))) => n,
            _ => {
                // this should never happen
                // hi absurd GitHub search!
                i32::MAX
            }
        };

        let conn = DB_POOL.get()?;

        let issue_id = issue.select(id)
                            .filter(number.eq(issue_number))
                            .filter(repository.eq(repo))
                            .first::<i32>(&*conn)?;

        Ok(IssueComment {
               id: self.id,
               fk_issue: issue_id,
               fk_user: self.user.id,
               body: self.body.replace(0x00 as char, ""),
               created_at: self.created_at.naive_utc(),
               updated_at: self.updated_at.naive_utc(),
               repository: repo.to_string(),
           })
    }
}

#[derive(Debug, Deserialize)]
pub struct PullRequestFromJson {
    pub number: i32,
    pub review_comments_url: String,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub assignee: Option<GitHubUser>,
    pub milestone: Option<MilestoneFromJson>,
    pub locked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub commits: i32,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
}

impl PullRequestFromJson {
    pub fn with_repo(self, repo: &str) -> PullRequest {
        PullRequest {
            number: self.number,
            state: self.state.replace(0x00 as char, ""),
            title: self.title.replace(0x00 as char, ""),
            body: self.body.map(|s| s.replace(0x00 as char, "")),
            fk_assignee: self.assignee.map(|a| a.id),
            fk_milestone: self.milestone.map(|m| m.id),
            locked: self.locked,
            created_at: self.created_at.naive_utc(),
            updated_at: self.updated_at.naive_utc(),
            closed_at: self.closed_at.map(|t| t.naive_utc()),
            merged_at: self.merged_at.map(|t| t.naive_utc()),
            commits: self.commits,
            additions: self.additions,
            deletions: self.deletions,
            changed_files: self.changed_files,
            repository: repo.to_string(),
        }
    }
}

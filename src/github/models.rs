// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use std::collections::BTreeMap;
use std::convert::Into;
use std::i32;

use chrono::{DateTime, UTC};

use domain::github::{Issue, IssueComment, Milestone, PullRequest, GitHubUser};

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
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,
    pub closed_at: Option<DateTime<UTC>>,
    pub due_on: Option<DateTime<UTC>>,
}

impl Into<Milestone> for MilestoneFromJson {
    fn into(self) -> Milestone {
        Milestone {
            id: self.id,
            number: self.number,
            open: match &self.state as &str {
                "open" => true,
                _ => false,
            },
            title: self.title.replace(0x00 as char, ""),
            description: self.description.map(|s| s.replace(0x00 as char, "")),
            fk_creator: self.creator.id,
            open_issues: self.open_issues,
            closed_issues: self.closed_issues,
            created_at: self.created_at.naive_utc(),
            updated_at: self.updated_at.naive_utc(),
            closed_at: self.closed_at.map(|t| t.naive_utc()),
            due_on: self.due_on.map(|t| t.naive_utc()),
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
    pub closed_at: Option<DateTime<UTC>>,
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,
    pub comments_url: String,
}

impl Into<(Issue, Option<Milestone>)> for IssueFromJson {
    fn into(self) -> (Issue, Option<Milestone>) {
        let milestone_id = match self.milestone {
            Some(ref m) => Some(m.id),
            None => None,
        };

        let issue = Issue {
            number: self.number,
            fk_milestone: milestone_id,
            fk_user: self.user.id,
            fk_assignee: self.assignee.map(|a| a.id),
            open: match &*self.state {
                "open" => true,
                _ => false,
            },
            is_pull_request: self.pull_request.is_some(),
            title: self.title.replace(0x00 as char, ""),
            body: self.body.unwrap_or("".to_string()).replace(0x00 as char, ""),
            locked: self.locked,
            closed_at: self.closed_at.map(|t| t.naive_utc()),
            created_at: self.created_at.naive_utc(),
            updated_at: self.updated_at.naive_utc(),
            labels: match self.labels {
                Some(json_labels) => json_labels.into_iter().map(|l| l.name).collect(),
                None => vec![],
            },
        };

        (issue, self.milestone.map(|m| m.into()))
    }
}

#[derive(Debug, Deserialize)]
pub struct CommentFromJson {
    pub id: i32,
    pub html_url: String,
    pub body: String,
    pub user: GitHubUser,
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,
}

impl Into<IssueComment> for CommentFromJson {
    fn into(self) -> IssueComment {
        let issue_id = self.html_url
                           .split('#')
                           .next()
                           .map(|r| r.split('/').last().map(|n| n.parse::<i32>()));

        let issue_id = match issue_id {
            Some(Some(Ok(n))) => n,
            _ => {
                // this should never happen
                // hi absurd GitHub search!
                i32::MAX
            }
        };

        IssueComment {
            id: self.id,
            fk_issue: issue_id,
            fk_user: self.user.id,
            body: self.body.replace(0x00 as char, ""),
            created_at: self.created_at.naive_utc(),
            updated_at: self.updated_at.naive_utc(),
        }
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
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,
    pub closed_at: Option<DateTime<UTC>>,
    pub merged_at: Option<DateTime<UTC>>,
    pub commits: i32,
    pub additions: i32,
    pub deletions: i32,
    pub changed_files: i32,
}

impl Into<PullRequest> for PullRequestFromJson {
    fn into(self) -> PullRequest {
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
        }
    }
}

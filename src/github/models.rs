use std::collections::BTreeMap;
use std::convert::Into;
use std::u32;

use chrono::{DateTime, UTC};

use domain::github::{Issue, IssueComment, IssueLabel, Milestone, User};

#[derive(Debug, Deserialize)]
pub struct LabelFromJson {
    name: String,
    color: String,
}

#[derive(Debug, Deserialize)]
pub struct MilestoneFromJson {
    pub id: u32,
    pub number: u32,
    pub state: String,
    pub title: String,
    pub description: Option<String>,
    pub creator: User,
    pub open_issues: u32,
    pub closed_issues: u32,
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
            title: self.title,
            description: self.description,
            creator: self.creator,
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
pub struct IssueFromJson {
    pub number: u32,
    pub user: User,
    pub assignee: Option<User>,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub labels: Option<Vec<LabelFromJson>>,
    pub milestone: Option<MilestoneFromJson>,
    pub locked: bool,
    pub comments: u32,
    pub pull_request: Option<BTreeMap<String, String>>,
    pub closed_at: Option<DateTime<UTC>>,
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,
    pub comments_url: String,
}

impl IssueFromJson {
    pub fn into_labels(&self) -> Vec<IssueLabel> {
        let mut labels = vec![];

        if let Some(ref labels_from_json) = self.labels {
            for l in labels_from_json {
                labels.push(IssueLabel {
                    fk_issue: self.number,
                    label: l.name.clone(),
                    color: l.color.clone(),
                });
            }
        }

        labels
    }
}

impl Into<Issue> for IssueFromJson {
    fn into(self) -> Issue {
        Issue {
            number: self.number,
            fk_milestone: self.milestone.map(|m| m.number),
            fk_user: self.user.id,
            fk_assignee: self.assignee.map(|a| a.id),
            open: match &*self.state {
                "open" => true,
                _ => false,
            },
            title: self.title,
            body: self.body.unwrap_or("".to_string()),
            locked: self.locked,
            closed_at: self.closed_at.map(|t| t.naive_utc()),
            created_at: self.created_at.naive_utc(),
            updated_at: self.updated_at.naive_utc(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CommentFromJson {
    pub id: u32,
    pub html_url: String,
    pub body: String,
    pub user: User,
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,
}

impl Into<IssueComment> for CommentFromJson {
    fn into(self) -> IssueComment {
        let issue_id = self.html_url
                           .split('#')
                           .next()
                           .map(|r| r.split('/').last().map(|n| n.parse::<u32>()));

        let issue_id = match issue_id {
            Some(Some(Ok(n))) => n,
            _ => {
                // TODO log failed parsing
                u32::MAX
            }
        };

        IssueComment {
            id: self.id,
            fk_issue: issue_id,
            fk_user: self.user.id,
            body: self.body,
            created_at: self.created_at.naive_utc(),
            updated_at: self.updated_at.naive_utc(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReviewCommentFromJson {
    id: u32,
    diff_hunk: String,
    path: String,
    position: u32,
    original_position: u32,
    commit_id: String,
    original_commit_id: String,
    user: User,
    body: String,
    created_at: DateTime<UTC>,
    updated_at: DateTime<UTC>,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestFromJson {
    number: u32,
    review_comments_url: String,
    state: String,
    title: String,
    body: String,
    assignee: User,
    milestone: MilestoneFromJson,
    locked: bool,
    created_at: DateTime<UTC>,
    updated_at: DateTime<UTC>,
    closed_at: Option<DateTime<UTC>>,
    merged_at: Option<DateTime<UTC>>,
    commits: u32,
    additions: u32,
    deletions: u32,
    changed_files: u32,
}

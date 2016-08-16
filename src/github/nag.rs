use std::collections::BTreeSet;

use diesel::prelude::*;

use config::CONFIG;
use DB_POOL;
use domain::github::{GitHubUser, Issue, IssueComment, Membership, Team};
use domain::schema::*;
use error::*;

pub fn update_nags(mut comments: Vec<IssueComment>) -> DashResult<()> {

    // make sure we process the new comments in creation order
    comments.sort_by_key(|c| c.created_at);

    // let mut changed_rfcs = BTreeSet::new();

    for comment in &comments {

        if comment.body.starts_with(&CONFIG.rfc_bot_mention) {

            // don't accept bot commands from non-subteam members
            if !is_by_subteam_member(comment)? {
                continue;
            }

            // TODO check the nag (fcp merge/close/postpone/cancel, concern, resolve, reviewed, f?)

            // TODO if fcp merge/close/postpone/cancel, create/cancel the nag

            // TODO if fcp concern, add a new concern

            // TODO if fcp resolve, mark concern resolved


        } else {

            // TODO check to see if we need to complete any feedback requests

        }
    }

    // TODO after processing all concerns/resolves check to see if any FCPs are changed

    Ok(())
}

/// Check if an issue comment is written by a member of one of the subteams labelled on the issue.
fn is_by_subteam_member(comment: &IssueComment) -> DashResult<bool> {
    let conn = &*DB_POOL.get()?;

    let issue = issue::table.find(comment.fk_issue).first::<Issue>(conn)?;
    let user = githubuser::table.find(comment.fk_user).first::<GitHubUser>(conn)?;

    use domain::schema::memberships::dsl::*;

    let many_to_many = memberships.filter(fk_member.eq(user.id)).load::<Membership>(&*conn)?;

    for membership in many_to_many {
        let team = teams::table.find(membership.fk_team).first::<Team>(conn)?;

        if issue.labels.contains(&team.label) {
            return Ok(true);
        }
    }

    Ok(false)
}

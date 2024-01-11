use diesel::prelude::*;

use crate::domain::github::{GitHubUser, Issue, IssueComment};
use crate::domain::rfcbot::{FcpConcern, FcpProposal, FcpReviewRequest};
use crate::domain::schema::fcp_concern;
use crate::error::DashResult;
use crate::DB_POOL;

#[derive(Serialize)]
pub struct FcpWithInfo {
    pub fcp: FcpProposal,
    pub reviews: Vec<(GitHubUser, bool)>,
    // (Concern name, comment registering it, and user leaving it)
    pub concerns: Vec<(String, IssueComment, GitHubUser)>,
    pub issue: Issue,
    pub status_comment: IssueComment,
}

pub fn all_fcps() -> DashResult<Vec<FcpWithInfo>> {
    use crate::domain::schema::{
        fcp_proposal, fcp_review_request, githubuser, issue, issuecomment,
    };

    let conn = &*DB_POOL.get()?;

    let proposals = fcp_proposal::table
        .filter(fcp_proposal::fcp_start.is_null())
        .load::<FcpProposal>(conn)?;

    let mut all_fcps = Vec::new();

    for fcp in proposals {
        let reviews = fcp_review_request::table
            .filter(fcp_review_request::fk_proposal.eq(fcp.id))
            .load::<FcpReviewRequest>(conn)?;

        let raw_concerns = fcp_concern::table
            .filter(fcp_concern::fk_proposal.eq(fcp.id))
            .load::<FcpConcern>(conn)?;

        let mut concerns = Vec::new();

        for concern in raw_concerns {
            // Skip resolved concerns.
            if concern.fk_resolved_comment.is_some() {
                continue;
            }
            let user = githubuser::table
                .filter(githubuser::id.eq(concern.fk_initiator))
                .first(conn)?;
            let comment = issuecomment::table
                .filter(issuecomment::id.eq(concern.fk_initiating_comment))
                .first::<IssueComment>(conn)?;
            concerns.push((concern.name, comment, user));
        }

        let mut reviews_with_users = Vec::new();

        for review in reviews {
            let user = githubuser::table
                .filter(githubuser::id.eq(review.fk_reviewer))
                .first(conn)?;
            reviews_with_users.push((user, review.reviewed));
        }

        let status_comment = issuecomment::table
            .filter(issuecomment::id.eq(fcp.fk_bot_tracking_comment))
            .first::<IssueComment>(conn)?;

        let issue = issue::table
            .filter(issue::id.eq(fcp.fk_issue))
            .first::<Issue>(conn)?;

        let fcp_with_info = FcpWithInfo {
            fcp,
            reviews: reviews_with_users,
            concerns,
            issue,
            status_comment,
        };

        all_fcps.push(fcp_with_info);
    }

    Ok(all_fcps)
}

#[derive(Queryable, Serialize)]
pub struct IndividualFcp {
    issue: Issue,
    proposal: FcpProposal,
    review_request: FcpReviewRequest,
}

pub fn individual_nags(username: &str) -> DashResult<(GitHubUser, Vec<IndividualFcp>)> {
    use crate::domain::schema::{fcp_proposal, fcp_review_request, githubuser, issue};
    let conn = &*DB_POOL.get()?;

    let user = githubuser::table
        .filter(githubuser::login.eq(username))
        .first::<GitHubUser>(conn)?;

    let review_requests = fcp_review_request::table
        .inner_join(fcp_proposal::table)
        .filter(fcp_proposal::fcp_closed.eq(false))
        .filter(fcp_review_request::fk_reviewer.eq(user.id))
        .filter(fcp_review_request::reviewed.eq(false))
        .load::<(FcpReviewRequest, FcpProposal)>(conn)?;

    let mut fcps = Vec::new();
    for (rr, proposal) in review_requests {
        let issue = issue::table
            .filter(issue::id.eq(proposal.fk_issue))
            .first::<Issue>(conn)?;

        fcps.push(IndividualFcp {
            issue,
            proposal,
            review_request: rr,
        });
    }

    Ok((user, fcps))
}

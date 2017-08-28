use diesel::prelude::*;

use DB_POOL;
use domain::github::{GitHubUser, Issue, IssueComment};
use domain::rfcbot::{FcpProposal, FcpReviewRequest};
use error::DashResult;

#[derive(Serialize)]
pub struct FcpWithInfo {
    fcp: FcpProposal,
    reviews: Vec<(GitHubUser, bool)>,
    issue: Issue,
    status_comment: IssueComment,
}

pub fn all_fcps() -> DashResult<Vec<FcpWithInfo>> {
    use domain::schema::{fcp_proposal, fcp_review_request, githubuser, issue, issuecomment};

    let conn = &*DB_POOL.get()?;

    let proposals = fcp_proposal::table
        .filter(fcp_proposal::fcp_start.is_null())
        .load::<FcpProposal>(conn)?;

    let mut all_fcps = Vec::new();

    for fcp in proposals {
        let reviews = fcp_review_request::table
            .filter(fcp_review_request::fk_proposal.eq(fcp.id))
            .load::<FcpReviewRequest>(conn)?;

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
            fcp: fcp,
            reviews: reviews_with_users,
            issue: issue,
            status_comment: status_comment,
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
    use domain::schema::{fcp_proposal, fcp_review_request, githubuser, issue};
    let conn = &*DB_POOL.get()?;

    let user = githubuser::table
        .filter(githubuser::login.eq(username))
        .first::<GitHubUser>(conn)?;

    let review_requests = fcp_review_request::table
        .filter(fcp_review_request::fk_reviewer.eq(user.id))
        .filter(fcp_review_request::reviewed.eq(false))
        .load::<FcpReviewRequest>(conn)?;

    let mut fcps = Vec::new();

    for rr in review_requests {
        let proposal = fcp_proposal::table
            .filter(fcp_proposal::id.eq(rr.fk_proposal))
            .first::<FcpProposal>(conn)?;

        let issue = issue::table
            .filter(issue::id.eq(proposal.fk_issue))
            .first::<Issue>(conn)?;

        let fcp = IndividualFcp {
            issue: issue,
            proposal: proposal,
            review_request: rr,
        };

        fcps.push(fcp);
    }

    Ok((user, fcps))
}

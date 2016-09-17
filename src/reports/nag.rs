use diesel::expression::dsl::*;
use diesel::prelude::*;
use diesel::select;
use diesel::types::VarChar;

use DB_POOL;
use domain::github::{GitHubUser, Issue};
use domain::rfcbot::{FcpProposal, FcpReviewRequest};
use error::DashResult;

pub fn all_team_members() -> DashResult<Vec<String>> {
    let conn = try!(DB_POOL.get());

    // waiting on associations to get this into proper typed queries

    Ok(try!(select(sql::<VarChar>("\
        DISTINCT u.login \
        FROM githubuser u, memberships m \
        WHERE u.id = m.fk_member \
        ORDER BY u.login"))
        .load(&*conn)))
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

    let user = githubuser::table.filter(githubuser::login.eq(username)).first::<GitHubUser>(conn)?;

    let review_requests =
        fcp_review_request::table.filter(fcp_review_request::fk_reviewer.eq(user.id))
            .filter(fcp_review_request::reviewed.eq(false))
            .load::<FcpReviewRequest>(conn)?;

    let mut fcps = Vec::new();

    for rr in review_requests {
        let proposal = fcp_proposal::table.filter(fcp_proposal::id.eq(rr.fk_proposal))
            .first::<FcpProposal>(conn)?;

        let issue = issue::table.filter(issue::id.eq(proposal.fk_issue)).first::<Issue>(conn)?;

        let fcp = IndividualFcp {
            issue: issue,
            proposal: proposal,
            review_request: rr,
        };

        fcps.push(fcp);
    }

    Ok((user, fcps))
}

use diesel::prelude::*;
use diesel;

use config::RFC_BOT_MENTION;
use DB_POOL;
use domain::github::{GitHubUser, Issue, IssueComment, Membership, Team};
use domain::rfcbot::{FcpConcern, FcpProposal, FcpReviewRequest, NewFcpProposal, NewFcpConcern};
use domain::schema::*;
use error::*;

// TODO set up easy posting of comments to GitHub issues

pub fn update_nags(mut comments: Vec<IssueComment>) -> DashResult<()> {

    // make sure we process the new comments in creation order
    comments.sort_by_key(|c| c.created_at);

    // let mut changed_rfcs = BTreeSet::new();

    for comment in &comments {

        let (is_by_subteam_member, author, issue) = is_by_subteam_member(comment)?;

        // attempt to parse a command out of the comment
        if let Ok(command) = RfcBotCommand::from_str(&comment.body) {

            // don't accept bot commands from non-subteam members
            if !is_by_subteam_member {
                continue;
            }

            command.process(&author, &issue, comment)?;

        } else {
            resolve_applicable_feedback_requests(&author, &issue, comment)?;
        }
    }

    evaluate_nags()?;

    Ok(())
}

fn evaluate_nags() -> DashResult<()> {

    // TODO go through all open FCP proposals
    // TODO get associated concerns and reviews
    // TODO see if all concerns resolved and all subteam members reviewed

    Ok(())
}

fn resolve_applicable_feedback_requests(author: &GitHubUser,
                                        issue: &Issue,
                                        comment: &IssueComment)
                                        -> DashResult<()> {
    // TODO check for any open feedback requests, close them if no longer applicable

    Ok(())
}

/// Check if an issue comment is written by a member of one of the subteams labelled on the issue.
fn is_by_subteam_member(comment: &IssueComment) -> DashResult<(bool, GitHubUser, Issue)> {
    let conn = &*DB_POOL.get()?;

    let issue = issue::table.find(comment.fk_issue).first::<Issue>(conn)?;
    let user = githubuser::table.find(comment.fk_user).first::<GitHubUser>(conn)?;

    use domain::schema::memberships::dsl::*;

    let many_to_many = memberships.filter(fk_member.eq(user.id)).load::<Membership>(&*conn)?;

    for membership in many_to_many {
        let team = teams::table.find(membership.fk_team).first::<Team>(conn)?;

        if issue.labels.contains(&team.label) {
            return Ok((true, user, issue));
        }
    }

    Ok((false, user, issue))
}

#[derive(Debug, Eq, PartialEq)]
pub enum RfcBotCommand<'a> {
    FcpPropose(FcpDisposition),
    FcpCancel,
    Reviewed,
    NewConcern(&'a str),
    ResolveConcern(&'a str),
    FeedbackRequest(&'a str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FcpDisposition {
    Merge,
    Close,
    Postpone,
}

impl FcpDisposition {
    pub fn repr(self) -> &'static str {
        match self {
            FcpDisposition::Merge => "merge",
            FcpDisposition::Close => "close",
            FcpDisposition::Postpone => "postpone",
        }
    }
}

impl<'a> RfcBotCommand<'a> {
    pub fn process(self,
                   author: &GitHubUser,
                   issue: &Issue,
                   comment: &IssueComment)
                   -> DashResult<()> {

        let conn = &*DB_POOL.get()?;

        // check for existing FCP
        let existing_proposal = {
            use domain::schema::fcp_proposal::dsl::*;

            fcp_proposal.filter(fk_issue.eq(issue.id))
                .first::<FcpProposal>(conn)
                .optional()?
        };

        match self {
            RfcBotCommand::FcpPropose(disp) => {
                use domain::schema::fcp_proposal::dsl::*;

                if let Some(_) = existing_proposal {
                    // TODO if exists, either ignore or change disposition (pending feedback)

                } else {
                    // if not exists, create new FCP proposal with merge disposition
                    let proposal = NewFcpProposal {
                        fk_issue: issue.id,
                        fk_initiator: author.id,
                        fk_initiating_comment: comment.id,
                        disposition: disp.repr(),
                    };

                    diesel::insert(&proposal).into(fcp_proposal).execute(conn)?;

                    // TODO generate review requests for all relevant subteam members

                    // TODO leave github comment stating that FCP is proposed, ping reviewers
                }
            }
            RfcBotCommand::FcpCancel => {
                use domain::schema::fcp_proposal::dsl::*;

                if let Some(existing) = existing_proposal {
                    // if exists delete FCP with associated concerns, reviews, feedback requests
                    // db schema has ON DELETE CASCADE
                    diesel::delete(fcp_proposal.filter(id.eq(existing.id))).execute(conn)?;

                    // TODO leave github comment stating that FCP proposal cancelled

                } else {
                    // TODO if not exists, leave comment telling author they were wrong
                }
            }
            RfcBotCommand::Reviewed => {
                // set a reviewed entry for the comment author on this issue

                use domain::schema::fcp_review_request::dsl::*;

                if let Some(proposal) = existing_proposal {

                    let review_request = fcp_review_request
                        .filter(fk_proposal.eq(proposal.id))
                        .filter(fk_reviewer.eq(author.id))
                        .first::<FcpReviewRequest>(conn)
                        .optional()?;

                    if let Some(mut review_request) = review_request {
                        // store an FK to the comment marking for review (not null fk here means
                        // reviewed)
                        review_request.fk_reviewed_comment = Some(comment.id);

                        diesel::update(fcp_review_request.find(review_request.id))
                            .set(&review_request)
                            .execute(conn)?;
                    }

                } else {
                    // TODO post github comment letting reviewer know that no FCP proposal is active
                }
            }
            RfcBotCommand::NewConcern(concern_name) => {

                if let Some(proposal) = existing_proposal {
                    // check for existing concern
                    use domain::schema::fcp_concern::dsl::*;

                    let existing_concern = fcp_concern
                        .filter(fk_proposal.eq(proposal.id))
                        .filter(name.eq(concern_name))
                        .first::<FcpConcern>(conn)
                        .optional()?;

                    if let Some(_) = existing_concern {
                        // TODO if exists, leave comment with existing concerns
                    } else {
                        // if not exists, create new concern with this author as creator

                        let new_concern = NewFcpConcern {
                            fk_proposal: proposal.id,
                            fk_initiator: author.id,
                            fk_resolved_comment: None,
                            name: concern_name,
                        };

                        diesel::insert(&new_concern).into(fcp_concern).execute(conn)?;

                        // TODO post github comment with list of existing concerns
                    }

                } else {
                    // TODO post github comment letting concern initiator know no proposal active
                }

            }
            RfcBotCommand::ResolveConcern(concern_name) => {

                if let Some(proposal) = existing_proposal {
                    // check for existing concern
                    use domain::schema::fcp_concern::dsl::*;

                    let existing_concern = fcp_concern
                        .filter(fk_proposal.eq(proposal.id))
                        .filter(fk_initiator.eq(author.id))
                        .filter(name.eq(concern_name))
                        .first::<FcpConcern>(conn)
                        .optional()?;

                    if let Some(mut concern) = existing_concern {

                        // mark concern as resolved by adding resolved_comment
                        concern.fk_resolved_comment = Some(comment.id);

                        diesel::update(fcp_concern.find(concern.id))
                            .set(&concern)
                            .execute(conn)?;

                    } else {
                        // TODO if not exists, leave comment with existing concerns & authors
                    }

                } else {
                    // TODO post github comment letting concern initiator know no proposal active
                }
            }
            RfcBotCommand::FeedbackRequest(username) => {
                // TODO check for existing feedback request
                // TODO create feedback request
            }
        }

        Ok(())
    }

    pub fn from_str(command: &'a str) -> DashResult<RfcBotCommand<'a>> {

        if &command[..RFC_BOT_MENTION.len()] != RFC_BOT_MENTION {
            return Err(DashError::Misc);
        }

        // trim out the bot ping
        let command = command[RFC_BOT_MENTION.len() + 1..].trim();

        let mut tokens = command.split_whitespace();

        let invocation = tokens.next().ok_or(DashError::Misc)?;

        let first_line = command.lines().next().ok_or(DashError::Misc)?;

        match invocation {
            "fcp" => {
                let subcommand = tokens.next().ok_or(DashError::Misc)?;

                match subcommand {
                    "merge" => Ok(RfcBotCommand::FcpPropose(FcpDisposition::Merge)),
                    "close" => Ok(RfcBotCommand::FcpPropose(FcpDisposition::Close)),
                    "postpone" => Ok(RfcBotCommand::FcpPropose(FcpDisposition::Postpone)),
                    "cancel" => Ok(RfcBotCommand::FcpCancel),
                    _ => Err(DashError::Misc),
                }
            }
            "concern" => {

                let name_start = first_line.find("concern").unwrap() + "concern".len();

                Ok(RfcBotCommand::NewConcern(first_line[name_start..].trim()))
            }
            "resolved" => {

                let name_start = first_line.find("resolved").unwrap() + "resolved".len();

                Ok(RfcBotCommand::ResolveConcern(first_line[name_start..].trim()))

            }
            "reviewed" => Ok(RfcBotCommand::Reviewed),
            "f?" => {

                let user = tokens.next().ok_or(DashError::Misc)?;

                if user.len() == 0 {
                    return Err(DashError::Misc);
                }

                Ok(RfcBotCommand::FeedbackRequest(&user[1..]))
            }
            _ => Err(DashError::Misc),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn success_fcp_reviewed() {
        let body = "@rfcbot: reviewed";
        let body_no_colon = "@rfcbot reviewed";

        let with_colon = RfcBotCommand::from_str(body).unwrap();
        let without_colon = RfcBotCommand::from_str(body_no_colon).unwrap();

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::Reviewed);
    }

    #[test]
    fn success_fcp_merge() {
        let body = "@rfcbot: fcp merge\n\nSome justification here.";
        let body_no_colon = "@rfcbot fcp merge\n\nSome justification here.";

        let with_colon = RfcBotCommand::from_str(body).unwrap();
        let without_colon = RfcBotCommand::from_str(body_no_colon).unwrap();

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::FcpMerge);
    }

    #[test]
    fn success_fcp_close() {
        let body = "@rfcbot: fcp close\n\nSome justification here.";
        let body_no_colon = "@rfcbot fcp close\n\nSome justification here.";

        let with_colon = RfcBotCommand::from_str(body).unwrap();
        let without_colon = RfcBotCommand::from_str(body_no_colon).unwrap();

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::FcpClose);
    }

    #[test]
    fn success_fcp_postpone() {
        let body = "@rfcbot: fcp postpone\n\nSome justification here.";
        let body_no_colon = "@rfcbot fcp postpone\n\nSome justification here.";

        let with_colon = RfcBotCommand::from_str(body).unwrap();
        let without_colon = RfcBotCommand::from_str(body_no_colon).unwrap();

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::FcpPostpone);
    }

    #[test]
    fn success_fcp_cancel() {
        let body = "@rfcbot: fcp cancel\n\nSome justification here.";
        let body_no_colon = "@rfcbot fcp cancel\n\nSome justification here.";

        let with_colon = RfcBotCommand::from_str(body).unwrap();
        let without_colon = RfcBotCommand::from_str(body_no_colon).unwrap();

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::FcpCancel);
    }

    #[test]
    fn success_concern() {
        let body = "@rfcbot: concern CONCERN_NAME
someothertext
somemoretext

somemoretext";
        let body_no_colon = "@rfcbot concern CONCERN_NAME
someothertext
somemoretext

somemoretext";

        let with_colon = RfcBotCommand::from_str(body).unwrap();
        let without_colon = RfcBotCommand::from_str(body_no_colon).unwrap();

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::NewConcern("CONCERN_NAME"));
    }

    #[test]
    fn success_resolve() {
        let body = "@rfcbot: resolved CONCERN_NAME
someothertext
somemoretext

somemoretext";
        let body_no_colon = "@rfcbot resolved CONCERN_NAME
someothertext
somemoretext

somemoretext";

        let with_colon = RfcBotCommand::from_str(body).unwrap();
        let without_colon = RfcBotCommand::from_str(body_no_colon).unwrap();

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::ResolveConcern("CONCERN_NAME"));
    }

    #[test]
    fn success_feedback() {
        let body = "@rfcbot: f? @bob
someothertext
somemoretext

somemoretext";
        let body_no_colon = "@rfcbot f? @bob
someothertext
somemoretext

somemoretext";

        let with_colon = RfcBotCommand::from_str(body).unwrap();
        let without_colon = RfcBotCommand::from_str(body_no_colon).unwrap();

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::FeedbackRequest("bob"));
    }
}

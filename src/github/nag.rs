use diesel::prelude::*;
use diesel;

use config::RFC_BOT_MENTION;
use DB_POOL;
use domain::github::{GitHubUser, Issue, IssueComment, Membership, Team};
use domain::rfcbot::{FcpConcern, FcpProposal, FcpReviewRequest, FeedbackRequest, NewFcpProposal,
                     NewFcpConcern, NewFcpReviewRequest, NewFeedbackRequest};
use domain::schema::*;
use error::*;
use super::GH;

pub fn update_nags(mut comments: Vec<IssueComment>) -> DashResult<()> {
    let conn = &*DB_POOL.get()?;

    // make sure we process the new comments in creation order
    comments.sort_by_key(|c| c.created_at);

    for comment in &comments {

        let issue = issue::table.find(comment.fk_issue).first::<Issue>(conn)?;
        let author = githubuser::table.find(comment.fk_user).first::<GitHubUser>(conn)?;
        let subteam_members = subteam_members(&issue)?;

        // attempt to parse a command out of the comment
        if let Ok(command) = RfcBotCommand::from_str(&comment.body) {

            // don't accept bot commands from non-subteam members
            if subteam_members.iter().find(|&u| u == &author).is_none() {
                continue;
            }

            command.process(&author, &issue, comment, &subteam_members)?;

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

    use domain::schema::rfc_feedback_request::dsl::*;
    let conn = &*DB_POOL.get()?;

    // check for an open feedback request, close since no longer applicable
    let existing_request = rfc_feedback_request.filter(fk_requested.eq(author.id))
        .filter(fk_issue.eq(issue.id))
        .first::<FeedbackRequest>(conn)
        .optional()?;

    if let Some(mut request) = existing_request {
        request.fk_feedback_comment = Some(comment.id);
        diesel::update(rfc_feedback_request.find(request.id)).set(&request).execute(conn)?;
    }

    Ok(())
}

/// Check if an issue comment is written by a member of one of the subteams labelled on the issue.
fn subteam_members(issue: &Issue) -> DashResult<Vec<GitHubUser>> {
    use diesel::pg::expression::dsl::any;
    use domain::schema::{teams, memberships, githubuser};

    let conn = &*DB_POOL.get()?;

    // retrieve all of the teams tagged on this issue
    let team = teams::table.filter(teams::label.eq(any(&issue.labels))).load::<Team>(conn)?;

    let team_ids = team.into_iter().map(|t| t.id).collect::<Vec<_>>();

    // get all the members of those teams
    let members = memberships::table.filter(memberships::fk_team.eq(any(team_ids)))
        .load::<Membership>(conn)?;

    let member_ids = members.into_iter().map(|m| m.fk_member).collect::<Vec<_>>();

    // resolve each member into an actual user
    let users = githubuser::table.filter(githubuser::id.eq(any(member_ids)))
        .order(githubuser::login)
        .load::<GitHubUser>(conn)?;

    Ok(users)
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
                   comment: &IssueComment,
                   issue_subteam_members: &[GitHubUser])
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
                use domain::schema::fcp_review_request;

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

                    let proposal = diesel::insert(&proposal).into(fcp_proposal)
                        .get_result::<FcpProposal>(conn)?;

                    // generate review requests for all relevant subteam members
                    for member in issue_subteam_members {

                        // don't generate a review request for the person who initiated the FCP
                        if member == author {
                            continue;
                        }

                        let review_request = NewFcpReviewRequest {
                            fk_proposal: proposal.id,
                            fk_reviewer: member.id,
                            fk_reviewed_comment: None,
                        };

                        diesel::insert(&review_request).into(fcp_review_request::table)
                            .execute(conn)?;
                    }

                    // leave github comment stating that FCP is proposed, ping reviewers
                    let comment =
                        RfcBotComment::new(author,
                                           issue,
                                           CommentType::FcpProposed(disp, issue_subteam_members));
                    comment.post();
                }
            }
            RfcBotCommand::FcpCancel => {
                use domain::schema::fcp_proposal::dsl::*;

                if let Some(existing) = existing_proposal {
                    // if exists delete FCP with associated concerns, reviews, feedback requests
                    // db schema has ON DELETE CASCADE
                    diesel::delete(fcp_proposal.filter(id.eq(existing.id))).execute(conn)?;

                    // leave github comment stating that FCP proposal cancelled
                    let comment =
                        RfcBotComment::new(author, issue, CommentType::FcpProposalCancelled);
                    comment.post();

                } else {
                    // if not exists, leave comment telling author they were wrong
                    let comment = RfcBotComment::new(author, issue, CommentType::FcpNoProposal);
                    comment.post();
                }
            }
            RfcBotCommand::Reviewed => {
                // set a reviewed entry for the comment author on this issue

                use domain::schema::fcp_review_request::dsl::*;

                if let Some(proposal) = existing_proposal {

                    let review_request = fcp_review_request.filter(fk_proposal.eq(proposal.id))
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
                    // post github comment letting reviewer know that no FCP proposal is active
                    let comment = RfcBotComment::new(author, issue, CommentType::FcpNoProposal);
                    comment.post();
                }
            }
            RfcBotCommand::NewConcern(concern_name) => {

                if let Some(proposal) = existing_proposal {
                    // check for existing concern
                    use domain::schema::fcp_concern::dsl::*;

                    let existing_concern = fcp_concern.filter(fk_proposal.eq(proposal.id))
                        .filter(name.eq(concern_name))
                        .first::<FcpConcern>(conn)
                        .optional()?;

                    if let Some(_) = existing_concern {
                        // if exists, leave comment with existing concerns

                        let concerns_w_authors =
                            self.list_active_concerns_with_authors(proposal.id)?;

                        let comment = RfcBotComment::new(author, issue,
                            CommentType::FcpDuplicateConcern(&concerns_w_authors));
                        comment.post();

                    } else {
                        // if not exists, create new concern with this author as creator

                        let new_concern = NewFcpConcern {
                            fk_proposal: proposal.id,
                            fk_initiator: author.id,
                            fk_resolved_comment: None,
                            name: concern_name,
                        };

                        diesel::insert(&new_concern).into(fcp_concern).execute(conn)?;

                        let comment = RfcBotComment::new(author,
                                                         issue,
                                                         CommentType::FcpNewConcern(concern_name));
                        comment.post();
                    }

                } else {
                    RfcBotComment::new(author, issue, CommentType::FcpNoProposal).post();
                }

            }
            RfcBotCommand::ResolveConcern(concern_name) => {

                if let Some(proposal) = existing_proposal {
                    // check for existing concern
                    use domain::schema::fcp_concern::dsl::*;

                    let existing_concern = fcp_concern.filter(fk_proposal.eq(proposal.id))
                        .filter(fk_initiator.eq(author.id))
                        .filter(name.eq(concern_name))
                        .first::<FcpConcern>(conn)
                        .optional()?;

                    if let Some(mut concern) = existing_concern {

                        // mark concern as resolved by adding resolved_comment
                        concern.fk_resolved_comment = Some(comment.id);

                        diesel::update(fcp_concern.find(concern.id)).set(&concern)
                            .execute(conn)?;

                        // list all the remaining concerns
                        let concerns_w_authors =
                            self.list_active_concerns_with_authors(proposal.id)?;

                        RfcBotComment::new(author,
                                           issue,
                                           CommentType::FcpResolvedConcern(&concern,
                                                                           &concerns_w_authors))
                            .post();

                    } else {
                        // if not exists, leave comment with existing concerns & authors

                        let concerns_w_authors =
                            self.list_active_concerns_with_authors(proposal.id)?;

                        let comment =
                            RfcBotComment::new(author,
                                               issue,
                                               CommentType::FcpMissingConcern(&concerns_w_authors));
                        comment.post();
                    }

                } else {
                    let comment = RfcBotComment::new(author, issue, CommentType::FcpNoProposal);
                    comment.post();
                }
            }
            RfcBotCommand::FeedbackRequest(username) => {

                use domain::schema::githubuser;
                use domain::schema::rfc_feedback_request::dsl::*;

                // we'll just assume that this user exists...it's very unlikely that someone
                // will request feedback from a user who's *never* commented or committed
                // on/to a rust-lang* repo
                let requested_user = githubuser::table.filter(githubuser::login.eq(username))
                    .first::<GitHubUser>(conn)?;

                // check for existing feedback request
                let existing_request =
                    rfc_feedback_request.filter(fk_requested.eq(requested_user.id))
                        .filter(fk_issue.eq(issue.id))
                        .first::<FeedbackRequest>(conn)
                        .optional()?;

                if existing_request.is_none() {
                    // create feedback request

                    let new_request = NewFeedbackRequest {
                        fk_initiator: author.id,
                        fk_requested: requested_user.id,
                        fk_issue: issue.id,
                        fk_feedback_comment: None,
                    };

                    diesel::insert(&new_request).into(rfc_feedback_request).execute(conn)?;
                }
            }
        }

        Ok(())
    }

    fn list_active_concerns_with_authors(&self,
                                         proposal_id: i32)
                                         -> DashResult<Vec<(FcpConcern, GitHubUser)>> {
        use domain::schema::{fcp_concern, githubuser};

        let conn = &*DB_POOL.get()?;

        let concerns = fcp_concern::table.filter(fcp_concern::fk_proposal.eq(proposal_id))
            .filter(fcp_concern::fk_resolved_comment.is_null())
            .order(fcp_concern::name)
            .load::<FcpConcern>(conn)?;

        let mut w_authors = Vec::with_capacity(concerns.len());

        for c in concerns {
            let initiator = githubuser::table.filter(githubuser::id.eq(c.fk_initiator))
                .first::<GitHubUser>(conn)?;

            w_authors.push((c, initiator));
        }

        Ok(w_authors)
    }

    pub fn from_str(command: &'a str) -> DashResult<RfcBotCommand<'a>> {

        // TODO support commands on any line

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

struct RfcBotComment<'a> {
    author: &'a GitHubUser,
    repository: &'a str,
    issue_num: i32,
    comment_type: CommentType<'a>,
}

enum CommentType<'a> {
    FcpProposed(FcpDisposition, &'a [GitHubUser]),
    FcpProposalCancelled,
    FcpNoProposal,
    FcpNewConcern(&'a str),
    FcpDuplicateConcern(&'a [(FcpConcern, GitHubUser)]),
    FcpMissingConcern(&'a [(FcpConcern, GitHubUser)]),
    FcpResolvedConcern(&'a FcpConcern, &'a [(FcpConcern, GitHubUser)]),
}

impl<'a> RfcBotComment<'a> {
    fn new(command_author: &'a GitHubUser,
           issue: &'a Issue,
           comment_type: CommentType<'a>)
           -> RfcBotComment<'a> {

        RfcBotComment {
            author: command_author,
            repository: &issue.repository,
            issue_num: issue.number,
            comment_type: comment_type,
        }
    }

    fn format(&self) -> DashResult<String> {

        match self.comment_type {
            CommentType::FcpProposed(disposition, reviewers) => {
                let mut msg = format!("FCP proposed (with disposition to {}). Review requested from:

",
                                      disposition.repr());

                for member in reviewers {
                    msg.push_str("* @");
                    msg.push_str(&member.login);
                    msg.push('\n');
                }

                Ok(msg)
            }
            CommentType::FcpProposalCancelled => {
                Ok(format!("@{} FCP proposal cancelled.", &self.author.login))
            }
            CommentType::FcpNoProposal => {
                Ok(format!("@{} no FCP proposal is active on this issue.",
                           &self.author.login))
            }
            CommentType::FcpDuplicateConcern(concerns) => {
                let mut msg = format!("@{} this looks like a duplicate concern. Existing concerns:

",
                                      &self.author.login);

                self.build_list_of_concerns(&mut msg, concerns);

                Ok(msg)
            }
            CommentType::FcpMissingConcern(concerns) => {
                let mut msg = format!("@{} this doesn't look like an existing concern on this issue.

Existing concerns:

",
                                      &self.author.login);
                self.build_list_of_concerns(&mut msg, concerns);

                Ok(msg)
            }
            CommentType::FcpResolvedConcern(resolved, remaining_concerns) => {
                let mut msg = format!("@{} concern \"{}\" marked as resolved.

",
                                      &self.author.login,
                                      resolved.name);

                if remaining_concerns.len() > 0 {
                    self.build_list_of_concerns(&mut msg, remaining_concerns);
                } else {
                    msg.push_str("No remaining concerns currently registered.");
                }

                Ok(msg)
            }
            CommentType::FcpNewConcern(name) => Ok(format!("Added concern \"{}\".", name)),
        }
    }

    fn build_list_of_concerns(&self, msg: &mut String, concerns: &[(FcpConcern, GitHubUser)]) {
        for &(ref concern, ref originator) in concerns {
            msg.push_str("* ");
            msg.push_str(&concern.name);
            msg.push_str(" (from ");
            msg.push_str(&originator.login);
            msg.push_str(")\n");
        }
    }

    fn post(&self) {
        use config::CONFIG;

        if CONFIG.post_comments {

            let text = match self.format() {
                Ok(t) => t,
                Err(why) => {
                    error!("Problem formatting bot comment: {:?}", why);
                    return;
                }
            };

            match GH.new_comment(self.repository, self.issue_num, &text) {

                Ok(()) => {
                    info!("Posted a comment to {}#{}.",
                          self.repository,
                          self.issue_num)
                }

                Err(why) => {
                    error!("Unabled to post a comment to {}#{}: {:?}",
                           self.repository,
                           self.issue_num,
                           why)
                }
            }

        } else {
            info!("Skipping comment to {}#{}, comment posts are disabled.",
                  self.repository,
                  self.issue_num);
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

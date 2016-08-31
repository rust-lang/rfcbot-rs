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

// TODO check if new subteam label added for existing proposals

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

        } else if author.login == "rfcbot" {
            // this is an updated comment from the bot itself

            // parse out each "reviewed" status for each user, then update them

            let statuses = comment.body
                .lines()
                .filter(|l| l.starts_with("* ["))
                .map(|l| {
                    let l = l.trim_left_matches("* [");
                    let reviewed = l.starts_with("x");

                    (reviewed, l.trim_left_matches("x] @").trim_left_matches(" ] @"))
                });

            for (is_reviewed, username) in statuses {
                let user: GitHubUser = githubuser::table.filter(githubuser::login.eq(username))
                    .first(conn)?;

                let proposal: FcpProposal =
                    fcp_proposal::table.filter(fcp_proposal::fk_issue.eq(issue.id)).first(conn)?;

                {
                    use domain::schema::fcp_review_request::dsl::*;
                    let mut review_request: FcpReviewRequest =
                        fcp_review_request.filter(fk_proposal.eq(proposal.id))
                            .filter(fk_reviewer.eq(user.id))
                            .first(conn)?;

                    review_request.reviewed = is_reviewed;
                    diesel::update(fcp_review_request.find(review_request.id)).set(&review_request)
                        .execute(conn)?;
                }
            }

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
    // TODO trigger update of all status comments
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

                if existing_proposal.is_none() {
                    // if not exists, create new FCP proposal

                    // leave github comment stating that FCP is proposed, ping reviewers
                    let gh_comment =
                        RfcBotComment::new(author, issue, CommentType::FcpProposed(disp, &[], &[]));

                    let gh_comment_id = gh_comment.post(None)?;

                    let proposal = NewFcpProposal {
                        fk_issue: issue.id,
                        fk_initiator: author.id,
                        fk_initiating_comment: comment.id,
                        disposition: disp.repr(),
                        fk_bot_tracking_comment: gh_comment_id,
                    };

                    let proposal = diesel::insert(&proposal).into(fcp_proposal)
                        .get_result::<FcpProposal>(conn)?;

                    // generate review requests for all relevant subteam members

                    let review_requests = issue_subteam_members.iter()
                        .map(|member| {
                            NewFcpReviewRequest {
                                fk_proposal: proposal.id,
                                fk_reviewer: member.id,
                                reviewed: false,
                            }
                        })
                        .collect::<Vec<_>>();

                    diesel::insert(&review_requests).into(fcp_review_request::table)
                        .execute(conn)?;

                    // TODO we have all of the review requests, generate a new comment and post it

                    let gh_comment =
                        RfcBotComment::new(author, issue, CommentType::FcpProposed(disp, &[], &[]));

                    gh_comment.post(Some(gh_comment_id))?;
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
                    let _ = comment.post(None);

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
                        review_request.reviewed = true;

                        diesel::update(fcp_review_request.find(review_request.id))
                            .set(&review_request)
                            .execute(conn)?;
                    }

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

                    if existing_concern.is_none() {
                        // if not exists, create new concern with this author as creator

                        let new_concern = NewFcpConcern {
                            fk_proposal: proposal.id,
                            fk_initiator: author.id,
                            fk_resolved_comment: None,
                            name: concern_name,
                        };

                        diesel::insert(&new_concern).into(fcp_concern).execute(conn)?;
                    }

                }
            }
            RfcBotCommand::ResolveConcern(concern_name) => {

                debug!("Command is to resolve a concern ({}).", concern_name);

                if let Some(proposal) = existing_proposal {
                    // check for existing concern
                    use domain::schema::fcp_concern::dsl::*;

                    let existing_concern = fcp_concern.filter(fk_proposal.eq(proposal.id))
                        .filter(fk_initiator.eq(author.id))
                        .filter(name.eq(concern_name))
                        .first::<FcpConcern>(conn)
                        .optional()?;

                    if let Some(mut concern) = existing_concern {

                        debug!("Found a matching concern ({})", concern_name);

                        // mark concern as resolved by adding resolved_comment
                        concern.fk_resolved_comment = Some(comment.id);

                        diesel::update(fcp_concern.find(concern.id)).set(&concern)
                            .execute(conn)?;
                    }

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
                                         -> DashResult<Vec<(GitHubUser, FcpConcern)>> {
        use domain::schema::{fcp_concern, githubuser};

        let conn = &*DB_POOL.get()?;

        let concerns = fcp_concern::table.filter(fcp_concern::fk_proposal.eq(proposal_id))
            .filter(fcp_concern::fk_resolved_comment.is_null())
            .order(fcp_concern::name)
            .load::<FcpConcern>(conn)?;

        let mut w_authors = Vec::with_capacity(concerns.len());

        for concern in concerns {
            let initiator = githubuser::table.filter(githubuser::id.eq(concern.fk_initiator))
                .first::<GitHubUser>(conn)?;

            w_authors.push((initiator, concern));
        }

        Ok(w_authors)
    }

    pub fn from_str(command: &'a str) -> DashResult<RfcBotCommand<'a>> {

        // get the tokens for the command line (starts with a bot mention)
        let command = command.lines()
            .filter(|&l| l.starts_with(RFC_BOT_MENTION))
            .next()
            .ok_or(DashError::Misc)?
            .trim_left_matches(RFC_BOT_MENTION)
            .trim_left_matches(':')
            .trim();

        let mut tokens = command.split_whitespace();

        let invocation = tokens.next().ok_or(DashError::Misc)?;

        match invocation {
            "fcp" => {
                let subcommand = tokens.next().ok_or(DashError::Misc)?;

                debug!("Parsed command as new FCP proposal");

                match subcommand {
                    "merge" => Ok(RfcBotCommand::FcpPropose(FcpDisposition::Merge)),
                    "close" => Ok(RfcBotCommand::FcpPropose(FcpDisposition::Close)),
                    "postpone" => Ok(RfcBotCommand::FcpPropose(FcpDisposition::Postpone)),
                    "cancel" => Ok(RfcBotCommand::FcpCancel),
                    _ => Err(DashError::Misc),
                }
            }
            "concern" => {

                let name_start = command.find("concern").unwrap() + "concern".len();

                debug!("Parsed command as NewConcern");

                Ok(RfcBotCommand::NewConcern(command[name_start..].trim()))
            }
            "resolved" => {

                let name_start = command.find("resolved").unwrap() + "resolved".len();

                debug!("Parsed command as ResolveConcern");

                Ok(RfcBotCommand::ResolveConcern(command[name_start..].trim()))

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
    FcpProposed(FcpDisposition,
                &'a [(GitHubUser, FcpReviewRequest)],
                &'a [(GitHubUser, FcpConcern)]),
    FcpProposalCancelled,
    FcpAllReviewedNoConcerns(&'a GitHubUser),
    FcpExpired(&'a GitHubUser),
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
            CommentType::FcpProposed(disposition, reviewers, concerns) => {
                let mut msg = String::from("FCP proposed with disposition to ");
                msg.push_str(disposition.repr());
                msg.push_str(". Review requested from:\n\n");

                for &(ref member, ref review_request) in reviewers {

                    if review_request.reviewed {
                        msg.push_str("* [x] @");
                    } else {
                        msg.push_str("* [ ] @");
                    }

                    msg.push_str(&member.login);
                    msg.push('\n');
                }

                if concerns.is_empty() {
                    msg.push_str("\nNo concerns currently listed.");
                } else {
                    msg.push_str("\nConcerns:\n\n");
                }

                for &(ref user, ref concern) in concerns {

                    if let Some(resolved_comment_id) = concern.fk_resolved_comment {
                        msg.push_str("* ~~");
                        msg.push_str(&concern.name);
                        msg.push_str("~~ (resolved https://github.com/");
                        msg.push_str(&self.repository);
                        msg.push_str("/issues/");
                        msg.push_str(&self.issue_num.to_string());
                        msg.push_str("#issuecomment-");
                        msg.push_str(&resolved_comment_id.to_string());
                        msg.push_str("\n");

                    } else {
                        msg.push_str("* ");
                        msg.push_str(&concern.name);
                        msg.push_str(" (@");
                        msg.push_str(&user.login);
                        msg.push_str(")\n");
                    }
                }

                Ok(msg)
            }
            CommentType::FcpProposalCancelled => {
                Ok(format!("@{} FCP proposal cancelled.", &self.author.login))
            }
            CommentType::FcpAllReviewedNoConcerns(initiator) => {
                Ok(format!("@{} all relevant subteam members have reviewed.
No concerns remain.",
                           initiator.login))
            }
            CommentType::FcpExpired(initiator) => {
                Ok(format!("@{} it has been one week since all blocks to the FCP were resolved.",
                           initiator.login))
            }
        }
    }

    fn post(&self, existing_comment: Option<i32>) -> DashResult<i32> {
        use config::CONFIG;

        if CONFIG.post_comments {

            let text = self.format()?;

            Ok(match existing_comment {
                    Some(comment_id) => GH.edit_comment(self.repository, comment_id, &text),
                    None => GH.new_comment(self.repository, self.issue_num, &text),
                }
                ?
                .id)

        } else {
            info!("Skipping comment to {}#{}, comment posts are disabled.",
                  self.repository,
                  self.issue_num);
            Err(DashError::Misc)
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
        assert_eq!(with_colon, RfcBotCommand::FcpPropose(FcpDisposition::Merge));
    }

    #[test]
    fn success_fcp_close() {
        let body = "@rfcbot: fcp close\n\nSome justification here.";
        let body_no_colon = "@rfcbot fcp close\n\nSome justification here.";

        let with_colon = RfcBotCommand::from_str(body).unwrap();
        let without_colon = RfcBotCommand::from_str(body_no_colon).unwrap();

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::FcpPropose(FcpDisposition::Close));
    }

    #[test]
    fn success_fcp_postpone() {
        let body = "@rfcbot: fcp postpone\n\nSome justification here.";
        let body_no_colon = "@rfcbot fcp postpone\n\nSome justification here.";

        let with_colon = RfcBotCommand::from_str(body).unwrap();
        let without_colon = RfcBotCommand::from_str(body_no_colon).unwrap();

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon,
                   RfcBotCommand::FcpPropose(FcpDisposition::Postpone));
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
    fn success_resolve_mid_body() {
        let body = "someothertext
@rfcbot: resolved CONCERN_NAME
somemoretext

somemoretext";
        let body_no_colon = "someothertext
somemoretext

@rfcbot resolved CONCERN_NAME

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

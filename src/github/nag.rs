use diesel::prelude::*;

use config::RFC_BOT_MENTION;
use DB_POOL;
use domain::github::{GitHubUser, Issue, IssueComment, Membership, Team};
use domain::schema::*;
use error::*;

pub fn update_nags(mut comments: Vec<IssueComment>) -> DashResult<()> {

    // make sure we process the new comments in creation order
    comments.sort_by_key(|c| c.created_at);

    // let mut changed_rfcs = BTreeSet::new();

    for comment in &comments {

        // attempt to parse a command out of the comment
        if let Ok(command) = RfcBotCommand::from_str(&comment.body) {

            let (is_by_subteam_member, author) = is_by_subteam_member(comment)?;

            // don't accept bot commands from non-subteam members
            if !is_by_subteam_member {
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
fn is_by_subteam_member(comment: &IssueComment) -> DashResult<(bool, GitHubUser)> {
    let conn = &*DB_POOL.get()?;

    let issue = issue::table.find(comment.fk_issue).first::<Issue>(conn)?;
    let user = githubuser::table.find(comment.fk_user).first::<GitHubUser>(conn)?;

    use domain::schema::memberships::dsl::*;

    let many_to_many = memberships.filter(fk_member.eq(user.id)).load::<Membership>(&*conn)?;

    for membership in many_to_many {
        let team = teams::table.find(membership.fk_team).first::<Team>(conn)?;

        if issue.labels.contains(&team.label) {
            return Ok((true, user));
        }
    }

    Ok((false, user))
}

#[derive(Debug, Eq, PartialEq)]
pub enum RfcBotCommand<'a> {
    FcpMerge,
    FcpClose,
    FcpPostpone,
    FcpCancel,
    Reviewed,
    NewConcern(&'a str),
    ResolveConcern(&'a str),
    FeedbackRequest(&'a str),
}

impl<'a> RfcBotCommand<'a> {
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
                    "merge" => Ok(RfcBotCommand::FcpMerge),
                    "close" => Ok(RfcBotCommand::FcpClose),
                    "postpone" => Ok(RfcBotCommand::FcpPostpone),
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
            "reviewed" => {
                Ok(RfcBotCommand::Reviewed)
            }
            "f?" => {

                let user = tokens.next().ok_or(DashError::Misc)?;

                if user.len() == 0 {
                    return Err(DashError::Misc);
                }

                Ok(RfcBotCommand::FeedbackRequest(&user[1..]))
            }
            _ => Err(DashError::Misc)
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

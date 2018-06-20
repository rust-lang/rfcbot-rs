use std::collections::BTreeSet;
use std::sync::Mutex;

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel;

use itertools::Itertools;

use DB_POOL;
use domain::github::{GitHubUser, Issue, IssueComment};
use domain::rfcbot::{FcpConcern, FcpProposal, FcpReviewRequest, FeedbackRequest, NewFcpProposal,
                     NewFcpConcern, NewFcpReviewRequest, NewFeedbackRequest,
                     NewPoll, Poll, NewPollReviewRequest, PollReviewRequest};
use domain::schema::*;
use error::*;
use github::models::CommentFromJson;
use teams::SETUP;
use super::GH;

use github::command::*;

impl Issue {
    fn remove_label(&self, label: Label) {
        let _ = GH.remove_label(&self.repository, self.number, label.as_str());
    }

    fn add_label(&self, label: Label) -> DashResult<()> {
        GH.add_label(&self.repository, self.number, label.as_str())
    }

    fn close(&self) {
        ok_or!(GH.close_issue(&self.repository, self.number), why =>
            error!("Unable to close issue {:?}: {:?}", self, why));
    }
}

lazy_static! {
    static ref NAG_LOCK: Mutex<()> = Mutex::new(());
}

// TODO check if new subteam label added for existing proposals

pub fn update_nags(comment: &IssueComment) -> DashResult<()> {
    let _in_progress_marker = NAG_LOCK.lock();

    let conn = &*DB_POOL.get()?;

    let issue = issue::table.find(comment.fk_issue).first::<Issue>(conn)?;

    let author = githubuser::table
        .find(comment.fk_user)
        .first::<GitHubUser>(conn)?;

    let subteam_members = subteam_members(&issue)?;

    // Attempt to parse all commands out of the comment
    let mut any = false;
    for command in RfcBotCommand::from_str_all(&comment.body) {
        any = true;

        // Don't accept bot commands from non-subteam members.
        // Early return because we'll just get here again...
        if subteam_members.iter().find(|&u| u == &author).is_none() {
            info!("command author ({}) doesn't appear in any relevant subteams",
                  author.login);
            return Ok(());
        }

        debug!("processing rfcbot command: {:?}", &command);
        let process = command.process(&author, &issue, comment, &subteam_members);
        ok_or!(process, why => {
            error!("Unable to process command for comment id {}: {:?}",
                comment.id, why);
            return Ok(());
        });

        debug!("rfcbot command is processed");
    }

    if !any {
        ok_or!(resolve_applicable_feedback_requests(&author, &issue, comment),
            why => error!("Unable to resolve feedback requests for comment id {}: {:?}",
                        comment.id, why));
    }

    ok_or!(evaluate_nags(), why =>
        error!("Unable to evaluate outstanding proposals: {:?}", why));

    Ok(())
}

fn update_proposal_review_status(proposal_id: i32) -> DashResult<()> {
    let conn = &*DB_POOL.get()?;
    // this is an updated comment from the bot itself

    // parse out each "reviewed" status for each user, then update them

    let proposal: FcpProposal = fcp_proposal::table.find(proposal_id).first(conn)?;

    // don't update any statuses if the fcp is running or closed
    if proposal.fcp_start.is_some() || proposal.fcp_closed {
        return Ok(());
    }

    let comment: IssueComment = issuecomment::table
        .find(proposal.fk_bot_tracking_comment)
        .first(conn)?;

    // parse the status comment and mark any new reviews as reviewed
    for username in parse_ticky_boxes("proposal", proposal.id, &comment) {
        let user: GitHubUser = githubuser::table
            .filter(githubuser::login.eq(username))
            .first(conn)?;

        {
            use domain::schema::fcp_review_request::dsl::*;
            let mut review_request: FcpReviewRequest = fcp_review_request
                .filter(fk_proposal.eq(proposal.id))
                .filter(fk_reviewer.eq(user.id))
                .first(conn)?;

            review_request.reviewed = true;
            diesel::update(fcp_review_request.find(review_request.id))
                .set(&review_request)
                .execute(conn)?;
        }
    }

    Ok(())
}

fn update_poll_review_status(poll_id: i32) -> DashResult<()> {
    let conn = &*DB_POOL.get()?;
    // this is an updated comment from the bot itself

    // parse out each "reviewed" status for each user, then update them

    let survey: Poll = poll::table.find(poll_id).first(conn)?;

    // don't update any statuses if the poll is closed
    if survey.poll_closed {
        return Ok(());
    }

    let comment: IssueComment = issuecomment::table
        .find(survey.fk_bot_tracking_comment)
        .first(conn)?;

    // parse the status comment and mark any new reviews as reviewed
    for username in parse_ticky_boxes("poll", survey.id, &comment) {
        let user: GitHubUser = githubuser::table
            .filter(githubuser::login.eq(username))
            .first(conn)?;

        {
            use domain::schema::poll_review_request::dsl::*;
            let mut review_request: PollReviewRequest = poll_review_request
                .filter(fk_poll.eq(survey.id))
                .filter(fk_reviewer.eq(user.id))
                .first(conn)?;

            review_request.reviewed = true;
            diesel::update(poll_review_request.find(review_request.id))
                .set(&review_request)
                .execute(conn)?;
        }
    }

    Ok(())
}

fn parse_ticky_boxes<'a>(what: &'a str, id: i32, comment: &'a IssueComment)
    -> impl Iterator<Item = &'a str>
{
    comment.body.lines().filter_map(move |line| if line.starts_with("* [") {
        let l = line.trim_left_matches("* [");
        let reviewed = l.starts_with('x');
        let remaining = l.trim_left_matches("x] @").trim_left_matches(" ] @");

        if let Some(username) = remaining.split_whitespace().next() {
            trace!("reviewer parsed as reviewed? {} (line: \"{}\")",
                    reviewed, l);

            if reviewed { Some(username) } else { None }
        } else {
            warn!("An empty usename showed up in comment {} for {} {}",
                  comment.id, what, id);
            None
        }
    } else {
        None
    })
}

fn evaluate_nags() -> DashResult<()> {
    evaluate_pendings()?;
    evaluate_ffcps()?;
    evaluate_polls()?;
    Ok(())
}

fn evaluate_polls() -> DashResult<()> {
    use domain::schema::poll::dsl::*;
    use domain::schema::issuecomment::dsl::*;
    use domain::schema::issuecomment::dsl::id as issuecomment_id;
    let conn = &*DB_POOL.get()?;

    // first process all "pending" polls (unreviewed)
    let pending = poll.filter(poll_closed.eq(false)).load::<Poll>(conn);
    let pending = ok_or!(pending, why => {
        error!("Unable to retrieve list of pending polls: {:?}", why);
        throw!(why)
    });

    for survey in pending {
        let initiator = githubuser::table.find(survey.fk_initiator)
                             .first::<GitHubUser>(conn);
        let initiator = ok_or_continue!(initiator, why =>
            error!("Unable to retrieve poll initiator for poll id {}: {:?}",
                    survey.id, why));

        let issue = issue::table.find(survey.fk_issue).first::<Issue>(conn);
        let issue = ok_or_continue!(issue, why =>
            error!("Unable to retrieve issue for poll {}: {:?}",
                    survey.id, why));
    
        // check to see if any checkboxes were modified before we end up replacing the comment
        ok_or_continue!(update_poll_review_status(survey.id), why =>
            error!("Unable to update review status for poll {}: {:?}",
                    survey.id, why));

        // get associated reviews
        let reviews = ok_or_continue!(list_poll_review_requests(survey.id), why =>
            error!("Unable to retrieve review requests for survey {}: {:?}",
                    survey.id, why));

        // update existing status comment with reviews & concerns
        let status_comment = RfcBotComment::new(&issue, CommentType::QuestionAsked {
            initiator: &initiator,
            reviewers: &reviews,
            question: &survey.poll_question,
            teams: survey.poll_teams.split(",").collect(),
        });

        let previous_comment: IssueComment = issuecomment
            .filter(issuecomment_id.eq(survey.fk_bot_tracking_comment))
            .first(conn)?;

        if previous_comment.body != status_comment.body {
            // if the comment body in the database equals the new one we generated, then no change
            // is needed from github (this assumes our DB accurately reflects GH's, which should
            // be true in most cases by the time this is called)
            let post = status_comment.post(Some(survey.fk_bot_tracking_comment));
            ok_or_continue!(post, why =>
                error!("Unable to update status comment for poll {}: {:?}",
                        survey.id, why));
        }
    }

    Ok(())
}

fn evaluate_pendings() -> DashResult<()> {
    use diesel::prelude::*;
    use domain::schema::fcp_proposal::dsl::*;
    use domain::schema::issuecomment::dsl::*;
    use domain::schema::issuecomment::dsl::id as issuecomment_id;
    let conn = &*DB_POOL.get()?;

    // first process all "pending" proposals (unreviewed or remaining concerns)
    let pending = fcp_proposal.filter(fcp_start.is_null()).load::<FcpProposal>(conn);
    let pending_proposals = ok_or!(pending, why => {
        error!("Unable to retrieve list of pending proposals: {:?}", why);
        throw!(why)
    });

    for mut proposal in pending_proposals {
        let initiator = githubuser::table.find(proposal.fk_initiator)
                             .first::<GitHubUser>(conn);
        let initiator = ok_or_continue!(initiator, why =>
            error!("Unable to retrieve proposal initiator for proposal id {}: {:?}",
                    proposal.id, why));

        let issue = issue::table.find(proposal.fk_issue).first::<Issue>(conn);
        let issue = ok_or_continue!(issue, why =>
            error!("Unable to retrieve issue for proposal {}: {:?}",
                    proposal.id, why));

        // if the issue has been closed before an FCP starts,
        // then we just need to cancel the FCP entirely
        if !issue.open {
            ok_or_continue!(cancel_fcp(&initiator, &issue, &proposal), why =>
                error!("Unable to cancel FCP for proposal {}: {:?}",
                        proposal.id, why));
        }

        // check to see if any checkboxes were modified before we end up replacing the comment
        ok_or_continue!(update_proposal_review_status(proposal.id), why =>
            error!("Unable to update review status for proposal {}: {:?}",
                    proposal.id, why));

        // get associated concerns and reviews
        let reviews = ok_or_continue!(list_review_requests(proposal.id), why =>
            error!("Unable to retrieve review requests for proposal {}: {:?}",
                    proposal.id, why));

        let concerns = ok_or_continue!(list_concerns_with_authors(proposal.id),
            why => error!("Unable to retrieve concerns for proposal {}: {:?}",
                    proposal.id, why));

        let num_outstanding_reviews = reviews.iter().filter(|&&(_, ref r)| !r.reviewed).count();
        let num_complete_reviews = reviews.len() - num_outstanding_reviews;
        let num_active_concerns = concerns
            .iter()
            .filter(|&&(_, ref c)| c.fk_resolved_comment.is_none())
            .count();

        // update existing status comment with reviews & concerns
        let status_comment = RfcBotComment::new(&issue, CommentType::FcpProposed(
                    &initiator,
                    FcpDisposition::from_str(&proposal.disposition)?,
                    &reviews,
                    &concerns));

        let previous_comment: IssueComment = issuecomment
            .filter(issuecomment_id.eq(proposal.fk_bot_tracking_comment))
            .first(conn)?;

        if previous_comment.body != status_comment.body {
            // if the comment body in the database equals the new one we generated, then no change
            // is needed from github (this assumes our DB accurately reflects GH's, which should
            // be true in most cases by the time this is called)
            let post = status_comment.post(Some(proposal.fk_bot_tracking_comment));
            ok_or_continue!(post, why =>
                error!("Unable to update status comment for proposal {}: {:?}",
                        proposal.id, why));
        }

        let majority_complete = num_outstanding_reviews < num_complete_reviews;

        if num_active_concerns == 0 && majority_complete && num_outstanding_reviews < 3 {
            // TODO only record the fcp as started if we know that we successfully commented
            // i.e. either the comment claims to have posted, or we get a comment back to reconcile

            // FCP can start now -- update the database
            proposal.fcp_start = Some(Utc::now().naive_utc());
            let update = diesel::update(fcp_proposal.find(proposal.id))
                          .set(&proposal).execute(conn);
            ok_or_continue!(update, why =>
                error!("Unable to mark FCP {} as started: {:?}",
                       proposal.id, why));

            // attempt to add the final-comment-period label
            // TODO only add label if FCP > 1 day
            use config::CONFIG;
            if CONFIG.post_comments {
                let label_res = issue.add_label(Label::FCP);
                issue.remove_label(Label::PFCP);
                let added_label = match label_res {
                    Ok(()) => true,
                    Err(why) => {
                        warn!("Unable to add FCP label to {}#{}: {:?}",
                              &issue.repository,
                              issue.number,
                              why);
                        false
                    }
                };

                let comment_type = CommentType::FcpAllReviewedNoConcerns {
                    added_label: added_label,
                    author: &initiator,
                    status_comment_id: proposal.fk_bot_tracking_comment,
                };

                // leave a comment for FCP start
                let fcp_start_comment = RfcBotComment::new(&issue, comment_type);
                ok_or_continue!(fcp_start_comment.post(None), why =>
                    error!("Unable to post comment for FCP {}'s start: {:?}",
                            proposal.id, why));
            }
        }
    }

    Ok(())
}

fn evaluate_ffcps() -> DashResult<()> {
    use diesel::prelude::*;
    use domain::schema::fcp_proposal::dsl::*;
    let conn = &*DB_POOL.get()?;

    // look for any FCP proposals that entered FCP a week or more ago but aren't marked as closed
    let one_business_week_ago = Utc::now().naive_utc() - Duration::days(10);
    let ffcps = fcp_proposal.filter(fcp_start.le(one_business_week_ago))
                            .filter(fcp_closed.eq(false))
                            .load::<FcpProposal>(conn);
    let finished_fcps = ok_or!(ffcps, why => {
        error!("Unable to retrieve FCPs that need to be marked as finished: {:?}",
               why);
        throw!(why);
    });

    for mut proposal in finished_fcps {
        let initiator = githubuser::table.find(proposal.fk_initiator)
                                         .first::<GitHubUser>(conn);
        let initiator = ok_or_continue!(initiator, why =>
            error!("Unable to retrieve proposal initiator for proposal id {}: {:?}",
                    proposal.id,
                    why));

        let issue = issue::table.find(proposal.fk_issue).first::<Issue>(conn);
        let issue = ok_or_continue!(issue, why =>
            error!("Unable to find issue to match proposal {}: {:?}",
                   proposal.id, why));

        // TODO only update the db if the comment posts, but reconcile if we find out it worked

        // update the fcp
        proposal.fcp_closed = true;
        let update_fcp = diesel::update(fcp_proposal.find(proposal.id))
                                .set(&proposal).execute(conn);
        ok_or_continue!(update_fcp, why =>
            error!("Unable to update FCP {}: {:?}", proposal.id, why));

        // parse the disposition:
        let disp = FcpDisposition::from_str(&proposal.disposition)?;

        // Add FFCP label and remove FCP label.
        let label_res = issue.add_label(Label::FFCP);
        issue.remove_label(Label::FCP);
        let added_label = match label_res {
            Ok(_) => true,
            Err(why) => {
                warn!("Unable to add Finished-FCP label to {}#{}: {:?}",
                        &issue.repository,
                        issue.number,
                        why);
                false
            }
        };

        // Build the comment:
        let comment_type = CommentType::FcpWeekPassed {
            added_label,
            author: &initiator,
            status_comment_id: proposal.fk_bot_tracking_comment,
            disposition: disp
        };
        let fcp_close_comment = RfcBotComment::new(&issue, comment_type);

        // Post it!
        ok_or_continue!(fcp_close_comment.post(None), why =>
            error!("Unable to post FCP-ending comment for proposal {}: {:?}",
                    proposal.id, why));

        execute_ffcp_actions(&issue, disp);
    }

    Ok(())
}

fn can_ffcp_close(issue: &Issue) -> bool {
    SETUP.should_ffcp_auto_close(&issue.repository)
}

fn can_ffcp_postpone(issue: &Issue) -> bool {
    SETUP.should_ffcp_auto_postpone(&issue.repository)
}

fn execute_ffcp_actions(issue: &Issue, disposition: FcpDisposition) {
    match disposition {
        FcpDisposition::Merge => {
            // TODO: This one will require a lot of work to
            // auto-merge RFCs and create the tracking issue.
        },
        FcpDisposition::Close if can_ffcp_close(issue) => {
            let _ = issue.add_label(Label::Closed);
            issue.remove_label(Label::DispositionClose);
            issue.close();
        },
        FcpDisposition::Postpone if can_ffcp_postpone(issue) => {
            let _ = issue.add_label(Label::Postponed);
            issue.remove_label(Label::DispositionPostpone);
            issue.close();
        },
        _ => {},
    }
}

fn list_review_requests(proposal_id: i32) -> DashResult<Vec<(GitHubUser, FcpReviewRequest)>> {
    use domain::schema::{fcp_review_request, githubuser};

    let conn = &*DB_POOL.get()?;

    let reviews = fcp_review_request::table
        .filter(fcp_review_request::fk_proposal.eq(proposal_id))
        .load::<FcpReviewRequest>(conn)?;

    let mut w_reviewers = Vec::with_capacity(reviews.len());

    for review in reviews {
        let initiator = githubuser::table
            .filter(githubuser::id.eq(review.fk_reviewer))
            .first::<GitHubUser>(conn)?;

        w_reviewers.push((initiator, review));
    }

    w_reviewers.sort_by(|a, b| a.0.login.cmp(&b.0.login));

    Ok(w_reviewers)
}

fn list_poll_review_requests(poll_id: i32)
    -> DashResult<Vec<(GitHubUser, PollReviewRequest)>>
{
    use domain::schema::{poll_review_request, githubuser};

    let conn = &*DB_POOL.get()?;

    let reviews = poll_review_request::table
        .filter(poll_review_request::fk_poll.eq(poll_id))
        .load::<PollReviewRequest>(conn)?;

    let mut w_reviewers = Vec::with_capacity(reviews.len());

    for review in reviews {
        let initiator = githubuser::table
            .filter(githubuser::id.eq(review.fk_reviewer))
            .first::<GitHubUser>(conn)?;

        w_reviewers.push((initiator, review));
    }

    w_reviewers.sort_by(|a, b| a.0.login.cmp(&b.0.login));

    Ok(w_reviewers)
}

fn list_concerns_with_authors(proposal_id: i32) -> DashResult<Vec<(GitHubUser, FcpConcern)>> {
    use domain::schema::{fcp_concern, githubuser};

    let conn = &*DB_POOL.get()?;

    let concerns = fcp_concern::table
        .filter(fcp_concern::fk_proposal.eq(proposal_id))
        .order(fcp_concern::name)
        .load::<FcpConcern>(conn)?;

    let mut w_authors = Vec::with_capacity(concerns.len());

    for concern in concerns {
        let initiator = githubuser::table
            .filter(githubuser::id.eq(concern.fk_initiator))
            .first::<GitHubUser>(conn)?;

        w_authors.push((initiator, concern));
    }

    Ok(w_authors)
}

fn resolve_applicable_feedback_requests(author: &GitHubUser,
                                        issue: &Issue,
                                        comment: &IssueComment)
                                        -> DashResult<()> {

    use domain::schema::rfc_feedback_request::dsl::*;
    let conn = &*DB_POOL.get()?;

    // check for an open feedback request, close since no longer applicable
    let existing_request = rfc_feedback_request
        .filter(fk_requested.eq(author.id))
        .filter(fk_issue.eq(issue.id))
        .first::<FeedbackRequest>(conn)
        .optional()?;

    if let Some(mut request) = existing_request {
        request.fk_feedback_comment = Some(comment.id);
        diesel::update(rfc_feedback_request.find(request.id))
            .set(&request)
            .execute(conn)?;
    }

    Ok(())
}

fn resolve_logins_to_users(member_logins: &Vec<&str>) -> DashResult<Vec<GitHubUser>> {
    use diesel::pg::expression::dsl::any;
    use domain::schema::githubuser;
    let conn = &*DB_POOL.get()?;

    // resolve each member into an actual user
    let users = githubuser::table
        .filter(githubuser::login.eq(any(member_logins)))
        .order(githubuser::login)
        .load::<GitHubUser>(conn)?;

    Ok(users)
}

/// Check if an issue comment is written by a member of one of the subteams
/// satisfying the given predicate.
fn specific_subteam_members<F>(included: F) -> DashResult<Vec<GitHubUser>>
where
    F: Fn(&String) -> bool
{
    resolve_logins_to_users(&SETUP.teams()
        .filter(|&(label, _)| included(&label.0))
        .flat_map(|(_, team)| team.member_logins())
        .collect::<BTreeSet<_>>()
        .into_iter() // diesel won't work with btreeset, and dedup has weird lifetime errors
        .collect::<Vec<_>>()
    )
}

/// Check if an issue comment is written by a member of one of the subteams
/// labelled on the issue.
fn subteam_members(issue: &Issue) -> DashResult<Vec<GitHubUser>> {
    // retrieve all of the teams tagged on this issue
    specific_subteam_members(|label| issue.labels.contains(label))
}

fn cancel_fcp(author: &GitHubUser, issue: &Issue, existing: &FcpProposal) -> DashResult<()> {
    use domain::schema::fcp_proposal::dsl::*;

    let conn = &*DB_POOL.get()?;

    // if exists delete FCP with associated concerns, reviews, feedback requests
    // db schema has ON DELETE CASCADE
    diesel::delete(fcp_proposal.filter(id.eq(existing.id)))
        .execute(conn)?;

    // leave github comment stating that FCP proposal cancelled
    let comment = RfcBotComment::new(issue, CommentType::FcpProposalCancelled(author));
    let _ = comment.post(None);
    &[Label::FCP,
      Label::PFCP,
      Label::DispositionMerge,
      Label::DispositionClose,
      Label::DispositionPostpone,
    ].iter().for_each(|&lab| issue.remove_label(lab));

    Ok(())
}

fn existing_proposal(issue: &Issue) -> DashResult<Option<FcpProposal>> {
    use domain::schema::fcp_proposal::dsl::*;
    let conn = &*DB_POOL.get()?;
    Ok(fcp_proposal
        .filter(fk_issue.eq(issue.id))
        .first::<FcpProposal>(conn)
        .optional()?)
}

fn post_insert_comment(issue: &Issue, comment: CommentType) -> DashResult<IssueComment> {
    use domain::schema::issuecomment;
    let conn = &*DB_POOL.get()?;

    let comment = RfcBotComment::new(issue, comment);
    let comment = comment.post(None)?;
    info!("Posted base comment to github, no reviewers listed yet");

    // at this point our new comment doesn't yet exist in the database, so
    // we need to insert it
    let comment = comment.with_repo(&issue.repository)?;
    if let Err(why) =
        diesel::insert(&comment)
                .into(issuecomment::table)
                .execute(conn) 
    {
        warn!("issue inserting new record, maybe received webhook for it: {:?}",
                why);
    }

    Ok(comment)
}

impl<'a> RfcBotCommand<'a> {
    pub fn process(self,
                   author: &GitHubUser,
                   issue: &Issue,
                   comment: &IssueComment,
                   team_members: &[GitHubUser])
                   -> DashResult<()> {
        use self::RfcBotCommand::*;
        match self {
            AskQuestion { teams, question } =>
                process_poll(author, issue, comment, question, teams),
            FcpPropose(disp) =>
                process_fcp_propose(author, issue, comment, team_members, disp),
            FcpCancel => process_fcp_cancel(author, issue),
            Reviewed => process_reviewed(author, issue),
            NewConcern(concern_name) =>
                process_new_concern(author, issue, comment, concern_name),
            ResolveConcern(concern_name) =>
                process_resolve_concern(author, issue, comment, concern_name),
            FeedbackRequest(username) =>
                process_feedback_request(author, issue, username),
        }
    }
}

fn process_poll
    (author: &GitHubUser, issue: &Issue, comment: &IssueComment,
     question: &str, teams: BTreeSet<&str>)
    -> DashResult<()>
{
    use domain::schema::poll::dsl::*;
    use domain::schema::poll_review_request;
    let conn = &*DB_POOL.get()?;

    let teams = if teams.is_empty() {
        SETUP.teams()
            .filter(|&(label, _)| issue.labels.contains(&label.0))
            .map(|(label, _)| &*label.0)
            .collect::<BTreeSet<_>>()
    } else {
        teams
    };
    let members = specific_subteam_members(|l| teams.contains(&**l))?;

    info!("adding a new question to issue.");

    // leave github comment stating that question is asked, ping reviewers
    let gh_comment = post_insert_comment(issue, CommentType::QuestionAsked {
        initiator: author,
        teams: teams.clone(),
        question,
        reviewers: &[],
    })?;

    let teams_str = teams.iter().cloned().intersperse(",").collect::<String>();
    let new_poll = NewPoll {
        fk_issue: issue.id,
        fk_initiator: author.id,
        fk_initiating_comment: comment.id,
        fk_bot_tracking_comment: gh_comment.id,
        poll_question: question,
        poll_created_at: Utc::now().naive_utc(),
        poll_closed: false,
        poll_teams: &*teams_str,
    };
    let new_poll = diesel::insert(&new_poll).into(poll).get_result::<Poll>(conn)?;

    debug!("poll inserted into the database");

    // generate review requests for all relevant subteam members

    let review_requests = members
        .iter()
        .map(|member| NewPollReviewRequest {
            fk_poll: new_poll.id,
            fk_reviewer: member.id,
            // let's assume the initiator has reviewed it
            reviewed: member.id == author.id,
        })
        .collect::<Vec<_>>();

    diesel::insert(&review_requests)
        .into(poll_review_request::table)
        .execute(conn)?;

    // they're in the database, but now we need them paired with githubuser

    let review_requests = list_poll_review_requests(new_poll.id)?;

    debug!("poll review requests inserted into the database");

    // we have all of the review requests, generate a new comment and post it

    let new_gh_comment = RfcBotComment::new(issue, CommentType::QuestionAsked {
        initiator: author,
        teams,
        question,
        reviewers: &*review_requests,
    });
    new_gh_comment.post(Some(gh_comment.id))?;

    debug!("github comment updated with poll reviewers");

    Ok(())
}

fn process_fcp_propose
    (author: &GitHubUser, issue: &Issue, comment: &IssueComment,
     team_members: &[GitHubUser], disp: FcpDisposition)
    -> DashResult<()>
{
    debug!("processing fcp proposal: {:?}", disp);
    use domain::schema::fcp_proposal::dsl::*;
    use domain::schema::fcp_review_request;

    if existing_proposal(issue)?.is_none() {
        let conn = &*DB_POOL.get()?;
        // if not exists, create new FCP proposal
        info!("proposal is a new FCP, creating...");

        // leave github comment stating that FCP is proposed, ping reviewers
        let gh_comment = post_insert_comment(issue,
            CommentType::FcpProposed(author, disp, &[], &[]))?;

        let proposal = NewFcpProposal {
            fk_issue: issue.id,
            fk_initiator: author.id,
            fk_initiating_comment: comment.id,
            fk_bot_tracking_comment: gh_comment.id,
            disposition: disp.repr(),
            fcp_start: None,
            fcp_closed: false,
        };
        let proposal = diesel::insert(&proposal)
            .into(fcp_proposal)
            .get_result::<FcpProposal>(conn)?;

        debug!("proposal inserted into the database");

        // generate review requests for all relevant subteam members

        let review_requests = team_members
            .iter()
            .map(|member| NewFcpReviewRequest {
                fk_proposal: proposal.id,
                fk_reviewer: member.id,
                // let's assume the initiator has reviewed it
                reviewed: member.id == author.id,
            })
            .collect::<Vec<_>>();

        diesel::insert(&review_requests)
            .into(fcp_review_request::table)
            .execute(conn)?;

        // they're in the database, but now we need them paired with githubuser

        let review_requests = list_review_requests(proposal.id)?;

        debug!("review requests inserted into the database");

        // we have all of the review requests, generate a new comment and post it

        let new_gh_comment = RfcBotComment::new(issue,
            CommentType::FcpProposed(author, disp, &review_requests, &[]));
        new_gh_comment.post(Some(gh_comment.id))?;
        debug!("github comment updated with reviewers");
    }

    Ok(())
}

fn process_fcp_cancel(author: &GitHubUser, issue: &Issue) -> DashResult<()> {
    if let Some(existing) = existing_proposal(issue)? {
        cancel_fcp(author, issue, &existing)?;
    }
    Ok(())
}

fn process_reviewed(author: &GitHubUser, issue: &Issue) -> DashResult<()> {
    // set a reviewed entry for the comment author on this issue
    if let Some(proposal) = existing_proposal(issue)? {
        use domain::schema::fcp_review_request::dsl::*;
        let conn = &*DB_POOL.get()?;

        let review_request = fcp_review_request
            .filter(fk_proposal.eq(proposal.id))
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

    Ok(())
}

fn process_new_concern
    (author: &GitHubUser, issue: &Issue, comment: &IssueComment,
     concern_name: &str)
    -> DashResult<()>
{
    if let Some(mut proposal) = existing_proposal(issue)? {
        // check for existing concern
        use domain::schema::fcp_concern::dsl::*;
        use domain::schema::fcp_proposal::dsl::*;
        let conn = &*DB_POOL.get()?;

        let existing_concern = fcp_concern
            .filter(fk_proposal.eq(proposal.id))
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
                fk_initiating_comment: comment.id,
            };
            diesel::insert(&new_concern).into(fcp_concern).execute(conn)?;

            // Take us out of FCP and back into PFCP if need be:
            if proposal.fcp_start.is_some() {
                // Update DB: FCP is not started anymore.
                proposal.fcp_start = None;
                let update = diesel::update(fcp_proposal.find(proposal.id))
                                    .set(&proposal)
                                    .execute(conn);
                ok_or!(update, why => {
                    error!("Unable to mark FCP {} as unstarted: {:?}", proposal.id, why);
                    return Ok(());
                });

                // Update labels:
                let _ = issue.add_label(Label::PFCP);
                issue.remove_label(Label::FCP);
            }
        }
    }

    Ok(())
}

fn process_resolve_concern
    (author: &GitHubUser, issue: &Issue, comment: &IssueComment, concern_name: &str)
    -> DashResult<()>
{
    debug!("Command is to resolve a concern ({}).", concern_name);

    if let Some(proposal) = existing_proposal(issue)? {
        // check for existing concern
        use domain::schema::fcp_concern::dsl::*;
        let conn = &*DB_POOL.get()?;

        let existing_concern = fcp_concern
            .filter(fk_proposal.eq(proposal.id))
            .filter(fk_initiator.eq(author.id))
            .filter(name.eq(concern_name))
            .first::<FcpConcern>(conn)
            .optional()?;

        if let Some(mut concern) = existing_concern {
            // mark concern as resolved by adding resolved_comment
            debug!("Found a matching concern ({})", concern_name);
            concern.fk_resolved_comment = Some(comment.id);
            diesel::update(fcp_concern.find(concern.id))
                .set(&concern)
                .execute(conn)?;
        }

    }

    Ok(())
}

fn process_feedback_request(author: &GitHubUser, issue: &Issue, username: &str)
    -> DashResult<()>
{
    use domain::schema::githubuser;
    use domain::schema::rfc_feedback_request::dsl::*;
    let conn = &*DB_POOL.get()?;

    // we'll just assume that this user exists...it's very unlikely that someone
    // will request feedback from a user who's *never* commented or committed
    // on/to a rust-lang* repo
    let requested_user = githubuser::table
        .filter(githubuser::login.eq(username))
        .first::<GitHubUser>(conn)?;

    // check for existing feedback request
    let existing_request = rfc_feedback_request
        .filter(fk_requested.eq(requested_user.id))
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

    Ok(())
}

struct RfcBotComment<'a> {
    issue: &'a Issue,
    body: String,
    comment_type: CommentType<'a>,
}

#[derive(Clone)]
enum CommentType<'a> {
    FcpProposed(&'a GitHubUser,
                FcpDisposition,
                &'a [(GitHubUser, FcpReviewRequest)],
                &'a [(GitHubUser, FcpConcern)]),
    FcpProposalCancelled(&'a GitHubUser),
    FcpAllReviewedNoConcerns {
        author: &'a GitHubUser,
        status_comment_id: i32,
        added_label: bool,
    },
    FcpWeekPassed {
        author: &'a GitHubUser,
        status_comment_id: i32,
        added_label: bool,
        disposition: FcpDisposition
    },
    QuestionAsked {
        initiator: &'a GitHubUser,
        reviewers: &'a [(GitHubUser, PollReviewRequest)],
        question: &'a str,
        teams: BTreeSet<&'a str>,
    },
}

impl<'a> RfcBotComment<'a> {
    fn new(issue: &'a Issue, comment_type: CommentType<'a>) -> RfcBotComment<'a> {

        let body = Self::format(issue, &comment_type);

        RfcBotComment {
            issue: issue,
            body: body,
            comment_type: comment_type,
        }
    }

    fn couldnt_add_label<'b>(msg: &mut String, author: &'b GitHubUser, label: Label) {
        msg.push_str("\n\n*psst @");
        msg.push_str(&author.login);
        msg.push_str(", I wasn't able to add the `");
        msg.push_str(label.as_str());
        msg.push_str("` label, please do so.*");
    }

    fn format(issue: &Issue, comment_type: &CommentType) -> String {
        match *comment_type {
            CommentType::QuestionAsked { initiator, reviewers, question, ref teams } => {
                let mut msg = String::from("Team member @");
                msg.push_str(&initiator.login);
                msg.push_str(" has asked teams: ");
                msg.extend(teams.iter().cloned().intersperse(", "));
                msg.push_str(", for consensus on: \n > ");
                msg.push_str(question);
                msg.push_str("\n\n");
                format_ticky_boxes(&mut msg,
                    reviewers.iter().map(|(m, rr)| (m, rr.reviewed)));
                msg
            }

            CommentType::FcpProposed(initiator, disposition, reviewers, concerns) => {
                let mut msg = String::from("Team member @");
                msg.push_str(&initiator.login);
                msg.push_str(" has proposed to ");
                msg.push_str(disposition.repr());
                msg.push_str(" this. The next step is review by the rest of the tagged ");
                msg.push_str("teams:\n\n");

                format_ticky_boxes(&mut msg,
                    reviewers.iter().map(|(m, rr)| (m, rr.reviewed)));

                if concerns.is_empty() {
                    msg.push_str("\nNo concerns currently listed.\n");
                } else {
                    msg.push_str("\nConcerns:\n\n");
                }

                for &(_, ref concern) in concerns {
                    if let Some(resolved_comment_id) = concern.fk_resolved_comment {
                        msg.push_str("* ~~");
                        msg.push_str(&concern.name);
                        msg.push_str("~~ resolved by ");
                        Self::add_comment_url(issue, &mut msg, resolved_comment_id);
                        msg.push_str("\n");

                    } else {
                        msg.push_str("* ");
                        msg.push_str(&concern.name);
                        msg.push_str(" (");
                        Self::add_comment_url(issue, &mut msg, concern.fk_initiating_comment);
                        msg.push_str(")\n");
                    }
                }

                msg.push_str("\nOnce a majority of reviewers approve (and none object), this will enter its final ");
                msg.push_str("comment period. If you spot a major issue that hasn't been raised ");
                msg.push_str("at any point in this process, please speak up!\n");

                msg.push_str("\nSee [this document](");
                msg.push_str("https://github.com/anp/rfcbot-rs/blob/master/README.md");
                msg.push_str(") for info about what commands tagged team members can give me.");

                msg
            }

            CommentType::FcpProposalCancelled(initiator) => {
                format!("@{} proposal cancelled.", initiator.login)
            }

            CommentType::FcpAllReviewedNoConcerns {
                author,
                status_comment_id,
                added_label,
            } => {
                let mut msg = String::new();

                msg.push_str(":bell: **This is now entering its final comment period**, ");
                msg.push_str("as per the [review above](");
                Self::add_comment_url(issue, &mut msg, status_comment_id);
                msg.push_str("). :bell:");

                if !added_label {
                    Self::couldnt_add_label(&mut msg, author, Label::FCP);
                }

                msg
            }

            CommentType::FcpWeekPassed {
                author,
                added_label,
                status_comment_id,
                disposition
            } => {
                let mut msg = String::new();
                msg.push_str("The final comment period, with a disposition to **");
                msg.push_str(disposition.repr());
                msg.push_str("**, as per the [review above](");
                Self::add_comment_url(issue, &mut msg, status_comment_id);
                msg.push_str("), is now **complete**.");

                match disposition {
                    FcpDisposition::Merge => {}
                    FcpDisposition::Close if can_ffcp_close(issue) => {
                        msg.push_str("\n\nBy the power vested in me by Rust, I hereby close this RFC.");
                    },
                    FcpDisposition::Postpone if can_ffcp_postpone(issue) => {
                        msg.push_str("\n\nBy the power vested in me by Rust, I hereby postpone this RFC.");
                    },
                    _ => {},
                }

                if !added_label {
                    Self::couldnt_add_label(&mut msg, author, Label::FFCP);
                }

                msg
            },
        }
    }

    fn add_comment_url(issue: &Issue, msg: &mut String, comment_id: i32) {
        let to_add = format!("https://github.com/{}/issues/{}#issuecomment-{}",
                             issue.repository,
                             issue.number,
                             comment_id);
        msg.push_str(&to_add);
    }

    fn maybe_add_pfcp_label(&self) {
        if let CommentType::FcpProposed(_, disposition, ..) = self.comment_type {
            let _ = self.issue.add_label(Label::PFCP);
            let _ = self.issue.add_label(disposition.label());
        }
    }

    fn post(&self, existing_comment: Option<i32>) -> DashResult<CommentFromJson> {
        use config::CONFIG;

        if CONFIG.post_comments {
            if self.issue.open {
                if let Some(comment_id) = existing_comment {
                    self.maybe_add_pfcp_label();
                    GH.edit_comment(&self.issue.repository, comment_id, &self.body)
                } else { 
                    GH.new_comment(&self.issue.repository, self.issue.number, &self.body)
                }
            } else {
                info!("Skipping comment to {}#{}, the issue is no longer open",
                      self.issue.repository,
                      self.issue.number);

                throw!(DashError::Misc(None))
            }
        } else {
            info!("Skipping comment to {}#{}, comment posts are disabled.",
                  self.issue.repository,
                  self.issue.number);
            throw!(DashError::Misc(None))
        }
    }
}

fn format_ticky_boxes<'a>
    (msg: &mut String, reviewers: impl Iterator<Item = (&'a GitHubUser, bool)>) {
    for (member, reviewed) in reviewers {
        msg.push_str(if reviewed { "* [x] @" } else { "* [ ] @" });
        msg.push_str(&member.login);
        msg.push('\n');
    }
}

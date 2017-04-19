use std::io::Read;

use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::mac::MacResult;
use crypto::sha1::Sha1;
use hex::FromHex;
use iron;
use serde_json;

use DB_POOL;
use config::CONFIG;
use error::DashResult;
use github::models::{CommentFromJson, IssueFromJson, PullRequestFromJson};
use github::{handle_comment, handle_issue, handle_pr};

/// signature for request
/// see [this document](https://developer.github.com/webhooks/securing/) for more information
header! {(XHubSignature, "X-Hub-Signature") => [String]}

/// name of Github event
/// see [this document](https://developer.github.com/webhooks/#events) for available types
header! {(XGithubEvent, "X-Github-Event") => [String]}

/// unique id for each delivery
header! {(XGithubDelivery, "X-Github-Delivery") => [String]}

pub fn handler(req: &mut iron::Request) -> iron::IronResult<iron::Response> {
    match inner_handler(req) {
        Ok(()) => (),
        Err(why) => error!("Error processing webhook: {:?}", why),
    }

    Ok(iron::Response::with((iron::status::Ok, "ok")))
}

fn inner_handler(req: &mut iron::Request) -> DashResult<()> {
    if let (Some(&XGithubEvent(ref event_name)),
            Some(&XGithubDelivery(ref delivery_id)),
            Some(&XHubSignature(ref signature))) =
        (req.headers.get::<XGithubEvent>(),
         req.headers.get::<XGithubDelivery>(),
         req.headers.get::<XHubSignature>()) {

        // unfortunately we need to read untrusted input before authenticating
        // b/c we need to sha the request payload
        let mut body = String::new();
        req.body.read_to_string(&mut body)?;

        let mut authenticated = false;

        for secret in &CONFIG.github_webhook_secrets {
            if authenticate(secret, &body, signature) {
                // once we know it's from github, we'll parse it

                authenticated = true;

                let payload = parse_event(event_name, &body)?;

                let full_event = Event {
                    delivery_id: delivery_id.to_owned(),
                    event_name: event_name.to_owned(),
                    payload: payload,
                };

                info!("Received valid webhook ({} id {})",
                      full_event.event_name,
                      full_event.delivery_id);

                authenticated_handler(full_event)?;
                break;
            }
        }

        if !authenticated {
            warn!("Received invalid webhook: {:?}", req);
        }
    }

    Ok(())
}

fn authenticate(secret: &str, payload: &str, signature: &str) -> bool {
    // https://developer.github.com/webhooks/securing/#validating-payloads-from-github
    let sans_prefix = signature[5..].as_bytes();
    match Vec::from_hex(sans_prefix) {
        Ok(sigbytes) => {
            let mut mac = Hmac::new(Sha1::new(), secret.as_bytes());
            mac.input(payload.as_bytes());
            // constant time comparison
            mac.result() == MacResult::new(&sigbytes)
        }
        Err(_) => false,
    }
}

fn parse_event(event_name: &str, body: &str) -> DashResult<Payload> {
    match event_name {
        "issue_comment" => {
            let payload = serde_json::from_str(body)?;
            Ok(Payload::IssueComment(payload))
        }

        "issues" => {
            let payload = serde_json::from_str(body)?;
            Ok(Payload::Issues(payload))
        }

        "pull_request" => {
            let payload = serde_json::from_str(body)?;
            Ok(Payload::PullRequest(payload))
        }

        "commit_comment" |
        "create" |
        "delete" |
        "deployment" |
        "deployment_status" |
        "fork" |
        "gollum" |
        "label" |
        "member" |
        "membership" |
        "milestone" |
        "organization" |
        "page_build" |
        "public" |
        "pull_request_review_comment" |
        "pull_request_review" |
        "push" |
        "repository" |
        "release" |
        "status" |
        "team" |
        "team_add" |
        "watch" => {
            info!("Received {} event, ignoring...", event_name);
            Ok(Payload::Unsupported)
        }

        _ => {
            warn!("Received unrecognized event {}, check GitHub's API to see what's updated.",
                  event_name);
            Ok(Payload::Unsupported)
        }
    }
}

fn authenticated_handler(event: Event) -> DashResult<()> {
    let conn = &*DB_POOL.get()?;

    match event.payload {
        Payload::Issues(issue_event) => {
            handle_issue(conn, issue_event.issue, &issue_event.repository.full_name)?;
        }

        Payload::PullRequest(pr_event) => {
            handle_pr(conn, pr_event.pull_request, &pr_event.repository.full_name)?;
        }

        Payload::IssueComment(comment_event) => {
            // possible race conditions if we get a comment hook before the issue one (or we
            // missed the issue one), so make sure the issue exists first

            if comment_event.action != "deleted" {
                // TODO handle deleted comments properly
                handle_issue(conn, comment_event.issue, &comment_event.repository.full_name)?;
                handle_comment(conn, comment_event.comment, &comment_event.repository.full_name)?;
            }
        }

        Payload::Unsupported => (),
    }

    Ok(())
}

#[derive(Debug)]
pub struct Event {
    delivery_id: String,
    event_name: String,
    payload: Payload,
}

#[derive(Debug)]
pub enum Payload {
    Issues(IssuesEvent),
    IssueComment(IssueCommentEvent),
    PullRequest(PullRequestEvent),

    Unsupported,
}

#[derive(Debug, Deserialize)]
pub struct IssuesEvent {
    action: String,
    issue: IssueFromJson,
    repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct IssueCommentEvent {
    action: String,
    issue: IssueFromJson,
    repository: Repository,
    comment: CommentFromJson,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
    action: String,
    repository: Repository,
    number: i32,
    pull_request: PullRequestFromJson,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    full_name: String,
}

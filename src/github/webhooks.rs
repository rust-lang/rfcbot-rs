use std::io::Read;

use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::mac::MacResult;
use crypto::sha1::Sha1;
use hex::FromHex;
use rocket::http::Status;
use rocket::data::{self, Data, FromData};
use rocket::request::Request;
use rocket::outcome::Outcome::*;
use serde_json;

use config::CONFIG;
use error::{DashError, DashResult};
use github::models::{CommentFromJson, IssueFromJson, PullRequestFromJson};

#[derive(Debug)]
pub struct Event {
    pub delivery_id: String,
    pub event_name: String,
    pub payload: Payload,
}

impl FromData for Event {
    type Error = &'static str;
    fn from_data(request: &Request, data: Data) -> data::Outcome<Self, Self::Error> {
        let headers = request.headers();

        // see [this document](https://developer.github.com/webhooks/securing/) for more information
        let signature = match headers.get_one("X-Hub-Signature") {
            Some(s) => s,
            None => return Failure((Status::BadRequest, "missing signature header")),
        };

        // see [this document](https://developer.github.com/webhooks/#events) for available types
        let event_name = match headers.get_one("X-Github-Event") {
            Some(e) => e,
            None => return Failure((Status::BadRequest, "missing event header")),
        };

        // unique id for each delivery
        let delivery_id = match headers.get_one("X-Github-Delivery") {
            Some(d) => d,
            None => return Failure((Status::BadRequest, "missing delivery header")),
        };

        let mut body = String::new();
        if let Err(why) = data.open().read_to_string(&mut body) {
            error!("unable to read request body: {:?}", why);
            return Failure((Status::InternalServerError, "unable to read request body"));
        }

        for secret in &CONFIG.github_webhook_secrets {
            if authenticate(secret, &body, signature) {
                // once we know it's from github, we'll parse it

                let payload = match parse_event(event_name, &body) {
                    Ok(p) => p,
                    Err(DashError::Serde(why)) => {
                        info!("failed to parse webhook payload: {:?}", why);
                        return Failure((Status::BadRequest,
                                        "failed to deserialize request payload"));
                    }
                    Err(why) => {
                        error!("non-json-parsing error with webhook payload: {:?}", why);
                        return Failure((Status::InternalServerError,
                                        "unknown failure, check the logs"));
                    }
                };

                let full_event = Event {
                    delivery_id: delivery_id.to_owned(),
                    event_name: event_name.to_owned(),
                    payload: payload,
                };

                info!("Received valid webhook ({} id {})",
                      full_event.event_name,
                      full_event.delivery_id);

                return Success(full_event);
            }
        }

        warn!("Received invalid webhook: {:?}", request);
        warn!("Invalid webhook body: `{}`", body);
        warn!("Tried {} webhook secrets", CONFIG.github_webhook_secrets.len());
        Failure((Status::Forbidden, "unable to authenticate webhook"))
    }
}

fn authenticate(secret: &str, payload: &str, signature: &str) -> bool {
    // https://developer.github.com/webhooks/securing/#validating-payloads-from-github
    let sans_prefix = signature[5..].as_bytes();
    if let Ok(sigbytes) = Vec::from_hex(sans_prefix) {
        let mut mac = Hmac::new(Sha1::new(), secret.as_bytes());
        mac.input(payload.as_bytes());
        // constant time comparison
        mac.result() == MacResult::new(&sigbytes)
    } else {
        false
    }
}

fn parse_event(event_name: &str, body: &str) -> DashResult<Payload> {
    match event_name {
        "issue_comment" => Ok(Payload::IssueComment(serde_json::from_str(body)?)),
        "issues" => Ok(Payload::Issues(serde_json::from_str(body)?)),
        "pull_request" => Ok(Payload::PullRequest(serde_json::from_str(body)?)),

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


#[derive(Debug)]
pub enum Payload {
    Issues(IssuesEvent),
    IssueComment(IssueCommentEvent),
    PullRequest(PullRequestEvent),

    Unsupported,
}

#[derive(Debug, Deserialize)]
pub struct IssuesEvent {
    pub action: String,
    pub issue: IssueFromJson,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct IssueCommentEvent {
    pub action: String,
    pub issue: IssueFromJson,
    pub repository: Repository,
    pub comment: CommentFromJson,
}

#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
    pub action: String,
    pub repository: Repository,
    pub number: i32,
    pub pull_request: PullRequestFromJson,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub full_name: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusEvent {
    pub commit: Commit,
    pub state: String,
    pub target_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Commit {
    pub committer: Committer,
}

#[derive(Debug, Deserialize)]
pub struct Committer {
    pub login: String,
}

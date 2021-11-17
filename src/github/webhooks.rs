use std::io::Read;

use hex::FromHex;
use openssl::{hash::MessageDigest, memcmp, pkey::PKey, sign::Signer};
use rocket::data::{self, Data, FromDataSimple};
use rocket::http::Status;
use rocket::outcome::Outcome::*;
use rocket::request::Request;

use crate::config::CONFIG;
use crate::error::{DashError, DashResult};
use crate::github::models::{CommentFromJson, IssueFromJson, PullRequestFromJson};

#[derive(Debug)]
pub struct Event {
    pub delivery_id: String,
    pub event_name: String,
    pub payload: Payload,
}

impl FromDataSimple for Event {
    type Error = &'static str;
    fn from_data(request: &Request<'_>, data: Data) -> data::Outcome<Self, Self::Error> {
        let headers = request.headers();

        // see [this document](https://developer.github.com/webhooks/securing/) for more information
        let signature = match headers.get_one("X-Hub-Signature-256") {
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
            if authenticate(secret, &body, signature).is_ok() {
                // once we know it's from github, we'll parse it

                let payload = match parse_event(event_name, &body) {
                    Ok(p) => p,
                    Err(DashError::Serde(why)) => {
                        error!("failed to parse webhook payload: {:?}", why);
                        return Failure((
                            Status::BadRequest,
                            "failed to deserialize request payload",
                        ));
                    }
                    Err(DashError::SerdePath(why)) => {
                        error!("failed to parse webhook payload: {:?}", why);
                        return Failure((
                            Status::BadRequest,
                            "failed to deserialize request payload",
                        ));
                    }
                    Err(why) => {
                        error!("non-json-parsing error with webhook payload: {:?}", why);
                        return Failure((
                            Status::InternalServerError,
                            "unknown failure, check the logs",
                        ));
                    }
                };

                let full_event = Event {
                    delivery_id: delivery_id.to_owned(),
                    event_name: event_name.to_owned(),
                    payload,
                };

                info!(
                    "Received valid webhook ({} id {})",
                    full_event.event_name, full_event.delivery_id
                );

                return Success(full_event);
            }
        }

        warn!("Received invalid webhook: {:?}", request);
        warn!("Invalid webhook body: `{}`", body);
        warn!(
            "Tried {} webhook secrets",
            CONFIG.github_webhook_secrets.len()
        );
        Failure((Status::Forbidden, "unable to authenticate webhook"))
    }
}

fn authenticate(secret: &str, payload: &str, signature: &str) -> Result<(), ()> {
    // https://developer.github.com/webhooks/securing/#validating-payloads-from-github
    let signature = signature.get("sha256=".len()..).ok_or(())?.as_bytes();
    let signature = Vec::from_hex(signature).map_err(|_| ())?;
    let key = PKey::hmac(secret.as_bytes()).map_err(|_| ())?;
    let mut signer = Signer::new(MessageDigest::sha256(), &key).map_err(|_| ())?;
    signer.update(payload.as_bytes()).map_err(|_| ())?;
    let hmac = signer.sign_to_vec().map_err(|_| ())?;
    // constant time comparison
    if memcmp::eq(&hmac, &signature) {
        Ok(())
    } else {
        Err(())
    }
}

macro_rules! from_json {
    ($b:expr) => {{
        let body = $b;
        let jd = &mut serde_json::Deserializer::from_str(&body);
        serde_path_to_error::deserialize(jd)
    }};
}

fn parse_event(event_name: &str, body: &str) -> DashResult<Payload> {
    match event_name {
        "issue_comment" => Ok(Payload::IssueComment(from_json!(body)?)),
        "issues" => Ok(Payload::Issues(from_json!(body)?)),
        "pull_request" => Ok(Payload::PullRequest(from_json!(body)?)),

        "commit_comment"
        | "create"
        | "delete"
        | "deployment"
        | "deployment_status"
        | "fork"
        | "gollum"
        | "label"
        | "member"
        | "membership"
        | "milestone"
        | "organization"
        | "page_build"
        | "public"
        | "pull_request_review_comment"
        | "pull_request_review"
        | "push"
        | "repository"
        | "release"
        | "status"
        | "team"
        | "team_add"
        | "watch" => {
            info!("Received {} event, ignoring...", event_name);
            Ok(Payload::Unsupported)
        }

        _ => {
            warn!(
                "Received unrecognized event {}, check GitHub's API to see what's updated.",
                event_name
            );
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

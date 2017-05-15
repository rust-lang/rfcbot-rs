use std::io::Read;

use chrono::{DateTime, UTC};
use diesel;
use diesel::prelude::*;
use hyper::Client;
use hyper::header::UserAgent;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use serde::de::DeserializeOwned;
use serde_json;

use DB_POOL;
use domain::builds::Build;
use error::DashResult;

header! { (Accept, "Accept") => [String] }

const ROOT_URL: &'static str = "https://api.travis-ci.org";
const UA: &'static str = "rusty-dash/0.0.0";

pub fn get_build(build: &str) -> DashResult<()> {
    let url = format!("{}/repos/rust-lang/rust/builds/{}", ROOT_URL, build);
    let response: ResponseFromJson = get(&url)?;
    let conn = &*DB_POOL.get()?;

    for job in response.jobs.iter() {
        if job.finished_at.is_none() || job.state == "cancelled" {
            continue
        }

        let duration = job.finished_at.unwrap()
            .signed_duration_since(job.started_at.unwrap());
        let b = Build {
            number: response.build.id,
            builder_name: get_builder_name(&job.config),
            successful: job.state == "passed",
            message: String::new(),
            duration_secs: Some(duration.num_seconds() as i32),
            start_time: job.started_at.map(|dt| dt.naive_utc()),
            end_time: job.finished_at.map(|dt| dt.naive_utc()),
        };

        {
            debug!("Inserting Travis build {:?}", b);
            use domain::schema::build::dsl::*;
            diesel::insert(&b).into(build).execute(conn)?;
        }
    }
    Ok(())
}

fn get<M: DeserializeOwned>(url: &str) -> DashResult<M> {
    let tls = NativeTlsClient::new().unwrap();
    let client = Client::with_connector(HttpsConnector::new(tls));
    let mut buffer = String::new();

    client.get(url)
        .header(UserAgent(UA.to_string()))
        .header(Accept("application/vnd.travis-ci.2+json".to_string()))
        .send()?
        .read_to_string(&mut buffer)?;
    match serde_json::from_str(&buffer) {
        Ok(m) => Ok(m),
        Err(reason) => {
            error!("Unable to parse Travis JSON: ({:?}): {}", reason, buffer);
            Err(reason.into())
        }
    }
}

fn get_builder_name(config: &ConfigFromJson) -> String {
    unimplemented!()
}

#[derive(Debug, Deserialize)]
struct ResponseFromJson{
    build: BuildFromJson,
    jobs: Vec<JobFromJson>
}

#[derive(Debug, Deserialize)]
struct BuildFromJson {
    id: i32,
}

#[derive(Debug, Deserialize)]
struct JobFromJson {
    state: String,
    config: ConfigFromJson,
    started_at: Option<DateTime<UTC>>,
    finished_at: Option<DateTime<UTC>>,
}

#[derive(Debug, Deserialize)]
struct ConfigFromJson {
    env: String,
    os: String,
}

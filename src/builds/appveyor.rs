use std::io::Read;

use chrono::{DateTime, UTC};
use diesel;
use diesel::prelude::*;
use hyper::Client;
use hyper::header::Accept;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use serde::de::DeserializeOwned;
use serde_json;

use DB_POOL;
use domain::builds::Build;
use error::DashResult;

static ROOT_URL: &'static str = "http://ci.appveyor.com/api";

pub fn get_and_insert_build(build: &str) -> DashResult<()> {
    let url = format!("{}/projects/rust-lang/rust/build/{}", ROOT_URL, build);
    let response: ResponseFromJson = get(&url)?;

    let conn = &*DB_POOL.get()?;
    for job in response.build.jobs.iter() {
        if job.status == "cancelled" {
            continue;
        }
        if let (Some(start), Some(end)) = (job.started, job.finished) {
            let duration = end.signed_duration_since(start);
            let b = Build {
                build_id: response.build.version.clone(),
                job_id: job.id.clone(),
                builder_name: "appveyor".to_string(),
                os: "windows".to_string(),
                env: job.name.clone(),
                successful: job.status == "success",
                message: job.status.to_owned(),
                duration_secs: Some(duration.num_seconds() as i32),
                start_time: Some(start.naive_utc()),
                end_time: Some(end.naive_utc()),
            };

            {
                debug!("Inserting Appveyor job {:?}", b);
                use domain::schema::build::dsl::*;
                diesel::insert(&b).into(build).execute(conn)?;
            }
        }
    }
    Ok(())
}

fn get<M: DeserializeOwned>(url: &str) -> DashResult<M> {
    let tls = NativeTlsClient::new().expect("Could not get TLS client");
    let client = Client::with_connector(HttpsConnector::new(tls));
    let mut buffer = String::new();
    client.get(url)
        .header(Accept::json())
        .send()?
        .read_to_string(&mut buffer)?;

    match serde_json::from_str(&buffer) {
        Ok(m) => Ok(m),
        Err(reason) => {
            error!("Unable to parse Appveyor JSON: ({:?}): {}", reason, buffer);
            Err(reason.into())
        }
    }
}

#[derive(Debug, Deserialize)]
struct ResponseFromJson {
    build: BuildFromJson,
}

#[derive(Debug, Deserialize)]
struct BuildFromJson {
    jobs: Vec<JobFromJson>,
    version: String,
}

#[derive(Debug, Deserialize)]
struct JobFromJson {
    #[serde(rename="jobId")]
    id: String,
    name: String,
    status: String,
    started: Option<DateTime<UTC>>,
    finished: Option<DateTime<UTC>>,
}

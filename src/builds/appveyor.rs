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

pub fn get_build(build: &str) -> DashResult<()> {
    let url = format!("{}/projects/rust-lang/rust/build/{}", ROOT_URL, build);
    let response: ResponseFromJson = get(&url)?;

    let conn = &*DB_POOL.get()?;
    for job in response.build.jobs.iter() {
        debug!("GOT {:?}", job);
        if job.finished.is_none() || job.status == "cancelled" {
            continue;
        }

        let duration = job.finished.unwrap().signed_duration_since(job.started);

        let b = Build {
            number: response.build.id,
            builder_name: get_builder_name(&job.name),
            successful: job.status == "success",
            message: job.status.to_owned(),
            duration_secs: Some(duration.num_seconds() as i32),
            start_time: Some(job.started.naive_utc()),
            end_time: job.finished.map(|dt| dt.naive_utc()),
        };

        {
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

fn get_builder_name(name: &str) -> String {
    unimplemented!()
}

#[derive(Debug, Deserialize)]
struct ResponseFromJson {
    build: BuildFromJson,
}

#[derive(Debug, Deserialize)]
struct BuildFromJson {
    #[serde(rename="buildId")]
    id: i32,
    jobs: Vec<JobFromJson>,
}

#[derive(Debug, Deserialize)]
struct JobFromJson {
    name: String,
    status: String,
    started: DateTime<UTC>,
    finished: Option<DateTime<UTC>>,
}

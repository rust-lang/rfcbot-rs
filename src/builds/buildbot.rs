use std::collections::BTreeMap;
use std::io::Read;

use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use serde_json;

use error::DashResult;
use DB_POOL;
use domain::builds::Build;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct BuildFromJson {
    number: i32,
    builderName: String,
    results: Option<i32>,
    times: (Option<f64>, Option<f64>),
    text: Vec<String>,
}

impl Into<Build> for BuildFromJson {
    fn into(self) -> Build {
        let successful = match self.results {
            Some(r) => r == 0,
            None => false,
        };

        let concat_msg = {
            let mut buf = String::new();
            for s in self.text {
                if !buf.is_empty() {
                    buf = buf + " ";
                }
                buf = buf + &s;
            }
            buf
        };

        let start_time = match self.times.0 {
            Some(t) => NaiveDateTime::from_timestamp_opt(t as i64, 0),
            None => None,
        };

        let end_time = match self.times.1 {
            Some(t) => NaiveDateTime::from_timestamp_opt(t as i64, 0),
            None => None,
        };

        let duration = if successful {
            match self.times {
                (Some(start), Some(end)) => Some((end - start) as i32),
                _ => None,
            }
        } else {
            None
        };

        let os = if self.builderName.contains("auto-win") {
            "windows"
        } else if self.builderName.contains("auto-linux") {
            "linux"
        } else if self.builderName.contains("auto-mac") {
            "osx"
        } else {
            "unknown"
        };

        Build {
            number: self.number,
            builder_name: "buildbot".to_string(),
            os: os.to_string(),
            env: self.builderName,
            successful: successful,
            message: concat_msg,
            duration_secs: duration,
            start_time: start_time,
            end_time: end_time,
        }
    }
}

pub fn ingest() -> DashResult<()> {
    info!("Ingesting buildbot data.");
    let conn = try!(DB_POOL.get());
    let c = Client::with_connector(HttpsConnector::new(NativeTlsClient::new().unwrap()));

    let mut resp = try!(c.get("https://buildbot.rust-lang.org/json/builders/").send());

    let mut buf = String::new();
    try!(resp.read_to_string(&mut buf));

    let builders = try!(serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&buf));
    let builders: Vec<_> = builders.into_iter()
        .map(|(b_id, _)| b_id)
        .collect();

    for builder in &builders {
        let url = format!("https://buildbot.rust-lang.org/json/builders/{}/builds/_all",
                          builder);
        debug!("GETing {}", &url);

        let mut resp = try!(c.get(&url).send());
        buf.clear();
        try!(resp.read_to_string(&mut buf));

        debug!("Parsing builds from JSON...");
        let builds = try!(serde_json::from_str::<BTreeMap<String, BuildFromJson>>(&buf));

        let builds = builds.into_iter()
            .filter(|&(_, ref b)| b.results.is_some())
            .map(|(_, b)| b.into())
            .collect::<Vec<Build>>();

        debug!("Inserting/updating records in database.");
        trace!("{:#?}", &builds);
        for b in builds {
            use domain::schema::build::dsl::*;
            let pk = build.select(id)
                .filter(number.eq(b.number))
                .filter(builder_name.eq(&b.builder_name))
                .first::<i32>(&*conn)
                .ok();

            if let Some(pk) = pk {
                try!(diesel::update(build.find(pk)).set(&b).execute(&*conn));
            } else {
                try!(diesel::insert(&b).into(build).execute(&*conn));
            }
        }
    }

    info!("Buildbot ingestion successful.");
    Ok(())
}

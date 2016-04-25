use std::collections::BTreeMap;
use std::io::Read;

use diesel;
use hyper::Client;
use serde_json;

use error::DashResult;
use DB_POOL;
use domain::buildbot::Build;

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

        let duration = if successful {
            match self.times {
                (Some(start), Some(end)) => Some((end - start) as i32),
                _ => None,
            }
        } else { None };

        let concat_msg = {
            let mut buf = String::new();
            for s in self.text {
                if buf.len() > 0 { buf = buf + " "; }
                buf = buf + &s;
            }
            buf
        };

        Build {
            number: self.number,
            builder_name: self.builderName,
            successful: successful,
            message: concat_msg,
            duration_secs: duration
        }
    }
}

pub fn ingest() -> DashResult<()> {
    let conn = try!(DB_POOL.get());
    let c = Client::new();

    let mut resp = try!(c.get("http://buildbot.rust-lang.org/json/builders/").send());

    let mut buf = String::new();
    try!(resp.read_to_string(&mut buf));

    let builders = try!(serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&buf));
    let builders: Vec<_> = builders.into_iter()
                                   .map(|(b_id, _)| b_id)
                                   .filter(|b_id| b_id.starts_with("auto-"))
                                   .collect();

    for builder in &builders {
        let url = format!("http://buildbot.rust-lang.org/json/builders/{}/builds/_all", builder);
        debug!("GETing {}", &url);

        let mut resp = try!(c.get(&url).send());
        buf.clear();
        try!(resp.read_to_string(&mut buf));

        debug!("Parsing builds from JSON...");
        let builds = try!(serde_json::from_str::<BTreeMap<String, BuildFromJson>>(&buf));

        let builds = builds.into_iter()
                           .filter(|&(_, ref b)| b.results.is_some())
                           .map(|(_, b)| b.into()).collect::<Vec<Build>>();

        debug!("Inserting/updating records in database.");
        for b in builds {
            use diesel::prelude::*;
            use domain::schema::build::dsl::*;
            let pk = build.filter(number.eq(b.number).and(builder_name.eq(&b.builder_name)))
                            .first::<(i32, i32, String, bool, String, Option<i32>)>(&*conn)
                            .map(|f| f.0)
                            .ok();

            if let Some(pk) = pk {
                try!(diesel::update(build.find(pk)).set(&b).execute(&*conn));
            } else {
                try!(diesel::insert(&b).into(build).execute(&*conn));
            }
        }
    }

    Ok(())
}

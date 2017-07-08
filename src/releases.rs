use chrono::{Datelike, DateTime, NaiveDate, Utc};
use diesel;
use diesel::expression::dsl::*;
use diesel::prelude::*;
use hyper::client::Client;
use hyper::net::HttpsConnector;
use hyper::status::StatusCode;
use hyper_native_tls::NativeTlsClient;

use DB_POOL;
use domain::releases::Release;
use error::DashResult;

lazy_static! {
    static ref CLIENT: Client = Client::with_connector(
        HttpsConnector::new(NativeTlsClient::new().unwrap()));
}

pub fn most_recent_update() -> DashResult<DateTime<Utc>> {
    info!("finding most recent nightly release updates");

    let conn = try!(DB_POOL.get());

    let most_recent: Option<NaiveDate> = {
        use domain::schema::release::dsl::*;
        try!(release.select(max(date)).filter(released).first(&*conn))
    };

    Ok(DateTime::from_utc(most_recent.expect("No releases found")
                          .and_hms(0, 0, 0), Utc))
}

fn get_release_for_date(d: NaiveDate) -> DashResult<Release> {
    let url = format!("https://static.rust-lang.org/dist/{}-{:02}-{:02}/channel-rust-nightly.toml",
                      d.year(),
                      d.month(),
                      d.day());

    let response = try!(CLIENT.get(&url).send());
    match response.status {
        StatusCode::Ok => Ok(Release { date: d, released: true }),
        _ => Ok(Release { date: d, released: false }),
    }
}

pub fn get_releases_since(d: NaiveDate) -> DashResult<Vec<Release>> {
    let mut releases = vec![];

    let mut curr = d;
    let today = Utc::today().naive_utc();

    while curr <= today {
        let curr_release = try!(get_release_for_date(curr));
        releases.push(curr_release);
        curr = curr.succ();
    }

    Ok(releases)
}

pub fn ingest_releases_since(d: DateTime<Utc>) -> DashResult<()> {
    use diesel::prelude::*;
    use domain::schema::release::dsl::*;
    let releases = try!(get_releases_since(d.date().naive_utc()));

    let conn = try!(DB_POOL.get());

    for r in releases {
        let pk = release.filter(date.eq(r.date))
            .first::<(i32, NaiveDate, bool)>(&*conn)
            .map(|f| f.0)
            .ok();

        if let Some(pk) = pk {
            try!(diesel::update(release.find(pk)).set(&r).execute(&*conn));
        } else {
            try!(diesel::insert(&r).into(release).execute(&*conn));
        }
    }

    Ok(())
}

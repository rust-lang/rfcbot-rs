use error::{DashResult, DashError};

mod appveyor;
mod travis;

use url::{Url, Host};


pub fn ingest_status_event(url: String) -> DashResult<()> {
    if let Ok(url) = Url::parse(&url) {
        if let Some(segments) = url.path_segments() {
            let build = segments.last().expect("segments guaranteed to have >=1 string");
            if let Some(Host::Domain(domain)) = url.host() {
                match domain {
                    "ci.appveyor.com" => return appveyor::get_and_insert_build(build),
                    "travis-ci.org" => return travis::get_and_insert_build(build),
                    _ => warn!("Unknown status URL found: {}", url),
                }
            }
        }
    }
    return Err(DashError::Misc(Some(format!("Could not parse URL {}", url))));
}

use error::{DashResult, DashError};

pub mod buildbot;
mod appveyor;

use url::{Url, Host};


pub fn ingest_status_event(url: String) -> DashResult<()> {
    if let Ok(url) = Url::parse(&url) {
        match (url.host(), url.path_segments()) {
            (Some(Host::Domain("ci.appveyor.com")), Some(segments)) => {
                return appveyor::get_build(segments.last().unwrap());
            },
            _ => return Err(DashError::Misc(Some(format!("Bad URL {}", url)))),
        }
    }
    return Err(DashError::Misc(Some(format!("Could not parse URL {}", url))));
}

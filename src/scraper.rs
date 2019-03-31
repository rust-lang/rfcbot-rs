use std::thread::JoinHandle;

use chrono::{DateTime, Utc};

use config::{CONFIG, GH_ORGS};
use github;

pub fn start_scraping() -> Option<JoinHandle<()>> {
    Some(::utils::spawn_thread("GitHub scraper", CONFIG.github_interval_mins?, || {
        scrape_github(github::most_recent_update()?);
        Ok(())
    }))
}

pub fn scrape_github(since: DateTime<Utc>) {
    let mut repos = Vec::new();
    for org in &GH_ORGS {
        repos.extend(ok_or!(github::GH.org_repos(org), why => {
            error!("Unable to retrieve repos for {}: {:?}", org, why);
            return;
        }));
    }

    info!("Scraping github activity since {:?}", since);
    let start_time = Utc::now().naive_utc();
    for repo in repos {
        match github::ingest_since(&repo, since) {
            Ok(_) => info!("Scraped {} github successfully", repo),
            Err(why) => error!("Unable to scrape github {}: {:?}", repo, why),
        }
    }

    ok_or!(github::record_successful_update(start_time), why =>
        error!("Problem recording successful update: {:?}", why));
}

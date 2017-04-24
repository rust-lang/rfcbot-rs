use std::thread;
use std::time::Duration;

use chrono::{DateTime, UTC};
use crossbeam::scope;

use config::{CONFIG, GH_ORGS};
use github;
use releases;

pub fn start_scraping() {
    // spawn the github scraper
    scope(|scope| {
        scope.spawn(|| {
            let sleep_duration = Duration::from_secs(CONFIG.github_interval_mins * 60);
            loop {
                match github::most_recent_update() {
                    Ok(gh_most_recent) => scrape_github(gh_most_recent),
                    Err(why) => error!("Unable to determine most recent GH update: {:?}", why)
                }
                info!("GitHub scraper sleeping for {} seconds ({} minutes)",
                      sleep_duration.as_secs(),
                      CONFIG.github_interval_mins);
                thread::sleep(sleep_duration);
            }
        });

        // spawn the nightly release scraper
        scope.spawn(|| {
            let sleep_duration = Duration::from_secs(CONFIG.release_interval_mins * 60);
            loop {
                if let Ok(rel_most_recent) = releases::most_recent_update() {
                    info!("scraping release activity since {:?}", rel_most_recent);
                    match releases::ingest_releases_since(rel_most_recent) {
                        Ok(()) => info!("scraped releases successfully"),
                        Err(why) => error!("unable to scrape releases: {:?}", why),
                    }
                } else {
                    error!("Unable to determine most recent release date.");
                }

                info!("Release scraper sleeping for {} seconds ({} minutes)",
                      sleep_duration.as_secs(),
                      CONFIG.release_interval_mins);
                thread::sleep(sleep_duration);
            }
        });
    });
}

pub fn scrape_github(since: DateTime<UTC>) {
    let mut repos = Vec::new();
    for org in &GH_ORGS {
        match github::GH.org_repos(org) {
            Ok(r) => repos.extend(r),
            Err(why) => {
                error!("Unable to retrieve repos for {}: {:?}", org, why);
                return;
            }
        }
    }

    info!("Scraping github activity since {:?}", since);
    let start_time = UTC::now().naive_utc();
    for repo in repos {
        match github::ingest_since(&repo, since) {
            Ok(()) => info!("Scraped {} github successfully", repo),
            Err(why) => error!("Unable to scrape github {}: {:?}", repo, why)
        }
    }

    match github::record_successful_update(start_time) {
        Ok(_) => {}
        Err(why) => error!("Problem recording successful update: {:?}", why)
    }
}

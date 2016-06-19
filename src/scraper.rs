use std::thread;
use std::time::Duration;

use crossbeam::scope;

use config::CONFIG;
use github;
use releases;
use buildbot;

pub fn start_scraping() {
    let orgs = ["rust-lang", "rust-lang-nursery", "rust-lang-deprecated"];

    // spawn the github scraper
    scope(|scope| {
        scope.spawn(|| {
            let sleep_duration = Duration::from_secs(CONFIG.github_interval_mins * 60);
            'outer: loop {
                let mut repos = Vec::new();

                for org in &orgs {
                    let r = github::GH.org_repos(org);

                    match r {
                        Ok(r) => repos.extend(r),
                        Err(why) => {
                            error!("Unable to retrieve repos for {}: {:?}", org, why);
                            info!("Sleeping for {} minutes", CONFIG.github_interval_mins);
                            thread::sleep(sleep_duration);
                            continue 'outer;
                        }
                    }
                }


                for repo in repos {

                    match github::most_recent_update(&repo) {
                        Ok(gh_most_recent) => {
                            info!("scraping github activity since {:?}", gh_most_recent);

                            match github::ingest_since(&repo, gh_most_recent) {
                                Ok(()) => info!("scraped github successfully"),
                                Err(why) => error!("unable to scrape github: {:?}", why),
                            }
                        }
                        Err(why) => {
                            error!("Unable to determine most recent github update ({:?}).", why)
                        }
                    }
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

        // spawn the buildbot scraper
        scope.spawn(|| {
            let sleep_duration = Duration::from_secs(CONFIG.buildbot_interval_mins * 60);
            loop {
                info!("scraping all buildbots...");
                match buildbot::ingest() {
                    Ok(()) => info!("scraped build status successfully"),
                    Err(why) => error!("unable to scrape build status: {:?}", why),
                }

                info!("Buildbot scraper sleeping for {} seconds ({} minutes)",
                      sleep_duration.as_secs(),
                      CONFIG.buildbot_interval_mins);
                thread::sleep(sleep_duration);
            }
        });
    });
}

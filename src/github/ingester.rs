// Copyright 2016 Adam Perry. Dual-licensed MIT and Apache 2.0 (see LICENSE files for details).

use chrono::{DateTime, UTC};

use config::CONFIG;
use github::{Client, GitHubResult};

lazy_static! {
    static ref GH: Client = Client::from(&*CONFIG);
}

pub fn ingest_since(start: DateTime<UTC>) -> GitHubResult<()> {
    println!("fetching all rust-lang/rust issues and comments since {}",
             start);
    let issues = GH.issues_since(start);
    let comments = GH.comments_since(start);

    if let (Ok(issues), Ok(comments)) = (issues, comments) {
        let mut prs = vec![];
        for issue in &issues {
            if let Some(ref pr_info) = issue.pull_request {
                match GH.fetch_pull_request(pr_info) {
                    Ok(pr) => prs.push(pr),
                    Err(why) => {
                        println!("ERROR fetching PR info: {:?}", why);
                        break;
                    }
                }
            }
        }

        println!("num pull requests updated since {}: {:#?}",
                 &start,
                 prs.len());

        println!("num issues updated since {}: {:?}", &start, issues.len());
        println!("num comments updated since {}: {:?}",
                 &start,
                 comments.len());
    } else {
        println!("ERROR retrieving issues and comments. You should probably add more error \
                  output.");
    }

    Ok(())
}

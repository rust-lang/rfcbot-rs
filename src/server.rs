use std::panic::catch_unwind;
use handlebars::Handlebars;
use rocket;

pub fn serve() {
    // in debug builds this will force an init, good enough for testing
    let _hbars = &*TEMPLATES;

    loop {
        let port = ::std::env::var("ROCKET_PORT").unwrap_or(String::from("OOPS"));
        info!("Attempting to launch Rocket at port {}...", &port);
        let result = catch_unwind(|| {
            rocket::ignite()
                .mount(
                    "/api",
                    routes![api::all_fcps, api::member_fcps, api::github_webhook],
                )
                .mount("/", routes![html::all_fcps, html::member_fcps])
                .launch();
        });

        ok_or!(result, why => error!("Rocket failed to ignite: {:?}", why));
    }
}

mod html {
    use std::collections::BTreeMap;
    use rocket::response::content;
    use super::TEMPLATES;
    use error::DashResult;
    use nag;

    #[get("/")]
    fn all_fcps() -> DashResult<content::Html<String>> {
        let mut teams = BTreeMap::new();
        for fcp in nag::all_fcps()? {
            let nag::FcpWithInfo {
                fcp,
                reviews,
                issue,
                status_comment,
            } = fcp;

            let mut pending_reviewers = reviews
                .into_iter()
                .filter(|&(_, reviewed)| !reviewed)
                .map(|(user, _)| user.login)
                .collect::<Vec<String>>();

            pending_reviewers.sort();

            let record = json!({
                "disposition": fcp.disposition,
                "issue": issue,
                "statusComment": status_comment,
                "pendingReviewers": pending_reviewers,
            });

            for label in issue.labels.iter().filter(|l| l.starts_with("T-")).cloned() {
                teams
                    .entry(label)
                    .or_insert_with(Vec::new)
                    .push(record.clone());
            }
        }

        let context = teams
            .into_iter()
            .map(|(team_label, fcps)| {
                json!({
                "team": team_label,
                "fcps": fcps,
            })
            })
            .collect::<Vec<_>>();

        let rendered = TEMPLATES.render("all", &json!({ "model": context }))?;
        Ok(content::Html(rendered))
    }

    #[get("/fcp/<username>")]
    fn member_fcps(username: String) -> DashResult<content::Html<String>> {
        let (user, fcps) = nag::individual_nags(&username)?;

        let context = json!({
            "model": {
                "user": user,
                "fcps": fcps,
            }
        });

        let rendered = TEMPLATES.render("user", &context)?;
        Ok(content::Html(rendered))
    }
}

mod api {
    use rocket_contrib::Json;
    use DB_POOL;
    use domain::github::GitHubUser;
    use error::DashResult;
    use github::{handle_comment, handle_issue, handle_pr};
    use github::webhooks::{Event, Payload};
    use nag;

    #[get("/all")]
    pub fn all_fcps() -> DashResult<Json<Vec<nag::FcpWithInfo>>> { Ok(Json(nag::all_fcps()?)) }

    #[get("/<username>")]
    pub fn member_fcps(
        username: String,
    ) -> DashResult<Json<(GitHubUser, Vec<nag::IndividualFcp>)>> {
        Ok(Json(nag::individual_nags(&username)?))
    }

    #[post("/github-webhook", data = "<event>")]
    pub fn github_webhook(event: Event) -> DashResult<()> {
        let conn = &*DB_POOL.get()?;

        match event.payload {
            Payload::Issues(issue_event) => {
                handle_issue(conn, issue_event.issue, &issue_event.repository.full_name)?;
            }

            Payload::PullRequest(pr_event) => {
                handle_pr(conn, pr_event.pull_request, &pr_event.repository.full_name)?;
            }

            Payload::IssueComment(comment_event) => {
                // possible race conditions if we get a comment hook before the issue one (or we
                // missed the issue one), so make sure the issue exists first

                if comment_event.action != "deleted" {
                    // TODO handle deleted comments properly
                    handle_issue(
                        conn,
                        comment_event.issue,
                        &comment_event.repository.full_name,
                    )?;
                    handle_comment(
                        conn,
                        comment_event.comment,
                        &comment_event.repository.full_name,
                    )?;
                }
            }

            Payload::Unsupported => (),
        }

        Ok(())
    }
}

lazy_static! {
    static ref TEMPLATES: Handlebars = {
        let mut hbars = Handlebars::new();
        let root_template = include_str!("templates/index.html");

        let all_fcps_fragment = include_str!("templates/fcp.hbs");
        let all_fcps_template = root_template.replace("{{content}}", all_fcps_fragment);

        let user_fcps_fragment = include_str!("templates/fcp-user.hbs");
        let user_fcps_template = root_template.replace("{{content}}", user_fcps_fragment);

        hbars.register_template_string("all", &all_fcps_template)
            .expect("unable to register all-fcps template");
        hbars.register_template_string("user", &user_fcps_template)
            .expect("unable to register user fcps template");

        hbars
    };
}

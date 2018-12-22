use std::panic::catch_unwind;
use rocket;
use rocket_contrib::templates::Template;

pub fn serve() {
    loop {
        let port = ::std::env::var("ROCKET_PORT").unwrap_or(String::from("OOPS"));
        info!("Attempting to launch Rocket at port {}...", &port);
        let result = catch_unwind(|| {
            rocket::ignite()
                .attach(Template::custom(|engines| {
                    let root_template = include_str!("templates/index.html");

                    let all_fcps_fragment = include_str!("templates/fcp.hbs");
                    let all_fcps_template = root_template.replace("{{content}}", all_fcps_fragment);

                    let user_fcps_fragment = include_str!("templates/fcp-user.hbs");
                    let user_fcps_template = root_template.replace("{{content}}", user_fcps_fragment);

                    engines.handlebars.register_template_string("all", &all_fcps_template)
                        .expect("unable to register all-fcps template");
                    engines.handlebars.register_template_string("user", &user_fcps_template)
                        .expect("unable to register user fcps template");
                }))
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
    use rocket_contrib::templates::Template;
    use error::DashResult;
    use nag;

    #[get("/")]
    pub fn all_fcps() -> DashResult<Template> {
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

        Ok(Template::render("all", &json!({ "model": context })))
    }

    #[get("/fcp/<username>")]
    pub fn member_fcps(username: String) -> DashResult<Template> {
        let (user, fcps) = nag::individual_nags(&username)?;

        let context = json!({
            "model": {
                "user": user,
                "fcps": fcps,
            }
        });

        Ok(Template::render("user", &context))
    }
}

mod api {
    use rocket_contrib::json::Json;
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

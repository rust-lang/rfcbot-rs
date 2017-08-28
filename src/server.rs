use rocket;



pub fn serve() {
    rocket::ignite()
        .mount("/api", routes![api::all_fcps, api::member_fcps])
        .mount("/", routes![html::all_fcps, html::member_fcps])
        .launch();
}

mod html {
    //use handlebars::Handlebars;
    use error::DashResult;
    // use nag;

    #[get("/")]
    fn all_fcps() -> DashResult<String> {
        //FIXME implement
        Ok(String::from(""))
    }

    #[get("/fcp/<username>")]
    fn member_fcps(username: String) -> DashResult<String> { Ok(String::from("")) }
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
    pub fn member_fcps(username: String)
                       -> DashResult<Json<(GitHubUser, Vec<nag::IndividualFcp>)>> {
        Ok(Json(nag::individual_nags(&username)?))
    }

    #[post("/github-webhook", data="<event>")]
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
                    handle_issue(conn,
                                 comment_event.issue,
                                 &comment_event.repository.full_name)?;
                    handle_comment(conn,
                                   comment_event.comment,
                                   &comment_event.repository.full_name)?;
                }
            }

            Payload::Unsupported => (),
        }

        Ok(())
    }
}

use std::thread::{spawn, JoinHandle};

use iron::prelude::*;
use iron::error::HttpResult;
use iron::{status, Listening};
use mount::Mount;
use router::Router;
use serde_json::ser;

use config::CONFIG;
use error::DashError;
use github::webhooks;
use nag;

pub fn serve() -> JoinHandle<HttpResult<Listening>> {
    let mut mount = Mount::new();

    mount.mount("/fcp/",
                router!(
        allfcps: get "/all" => list_fcps,
        usernamefcps: get "/:username" => member_nags
    ));

    mount.mount("/github-webhook", router!(ghwebhook: post "/" => webhooks::handler));

    // middleware goes here

    let server_addr = format!("0.0.0.0:{}", CONFIG.server_port);
    info!("Starting API server running at {}", &server_addr);
    spawn(move || { Iron::new(mount).http(&*server_addr) })
}

pub fn list_fcps(_: &mut Request) -> IronResult<Response> {
    let nag_report = nag::all_fcps()?;

    Ok(Response::with((status::Ok,
                       ser::to_string(&nag_report).map_err(|e| -> DashError {
                           e.into()
                       })?)))
}

pub fn member_nags(req: &mut Request) -> IronResult<Response> {
    let username = match req.extensions.get::<Router>().unwrap().find("username") {
        Some(u) => u,
        None => return Ok(Response::with((status::BadRequest, "Invalid team member username."))),
    };

    Ok(Response::with((status::Ok,
                       try!(ser::to_string(&try!(nag::individual_nags(username)))
                           .map_err(|e| {
                               let e: DashError = e.into();
                               e
                           })))))
}

use std::thread::{spawn, JoinHandle};

use iron::prelude::*;
use iron::error::HttpResult;
use iron::Listening;
use mount::Mount;

use config::CONFIG;
use github::webhooks;

mod handlers;

pub fn serve() -> JoinHandle<HttpResult<Listening>> {
    let mut mount = Mount::new();

    mount.mount("/fcp/",
                router!(
        allfcps: get "/all" => handlers::list_fcps,
        usernamefcps: get "/:username" => handlers::member_nags
    ));

    mount.mount("/github-webhook", router!(ghwebhook: post "/" => webhooks::handler));

    // middleware goes here

    let server_addr = format!("0.0.0.0:{}", CONFIG.server_port);
    info!("Starting API server running at {}", &server_addr);
    spawn(move || { Iron::new(mount).http(&*server_addr) })
}

use iron::prelude::*;
use mount::Mount;

use config::CONFIG;

mod handlers;

pub fn serve() {
    let mut mount = Mount::new();

    mount.mount("/pullrequests/",
                router!(get "/" => handlers::pull_requests));
    mount.mount("/issues/", router!(get "/" => handlers::issues));
    mount.mount("/buildbots/", router!(get "/" => handlers::buildbots));
    mount.mount("/releases/", router!(get "/" => handlers::releases));
    mount.mount("/hot-issues/", router!(get "/" => handlers::hot_issues));

    mount.mount("/fcp/",
                router!(
        get "/all" => handlers::list_fcps,
        get "/:username" => handlers::member_nags
    ));

    let chain = Chain::new(mount);

    // middleware goes here

    let server_addr = format!("0.0.0.0:{}", CONFIG.server_port);
    info!("Starting API server running at {}", &server_addr);
    Iron::new(chain).http(&*server_addr).unwrap();
}

use iron::prelude::*;
use mount::Mount;

use config::CONFIG;

mod handlers;

pub fn serve() {
    let mut mount = Mount::new();

    mount.mount("/pullrequests/", router!(get "/" => handlers::pull_requests));
    mount.mount("/issues/", router!(get "/" => handlers::issues));
    mount.mount("/buildbots/", router!(get "/" => handlers::buildbots));
    mount.mount("/releases/", router!(get "/" => handlers::releases));

    let chain = Chain::new(mount);

    //middleware goes here

    let server_addr = format!("0.0.0.0:{}", CONFIG.server_port);
    info!("Starting API server running at {}", &server_addr);
    Iron::new(chain).http(&*server_addr).unwrap();
}

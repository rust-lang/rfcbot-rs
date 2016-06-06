use iron::prelude::*;
use logger::Logger;
use mount::Mount;

use config::CONFIG;

mod handlers;

pub fn serve() {
    let mut mount = Mount::new();

    mount.mount("/pullrequests/", router!(get "/" => handlers::pull_requests));
    mount.mount("/issues/", router!(get "/" => handlers::issues));
    mount.mount("/buildbots/", router!(get "/" => handlers::buildbots));
    mount.mount("/releases/", router!(get "/" => handlers::releases)); 

    let mut chain = Chain::new(mount);

    let (logger_before, logger_after) = Logger::new(None);

    // Link logger_before as your first before middleware.
    chain.link_before(logger_before);

    // any new middlewares go here

    // Link logger_after as your *last* after middleware.
    chain.link_after(logger_after);

    let server_addr = format!("0.0.0.0:{}", CONFIG.server_port);
    info!("Starting API server running at {}", &server_addr);
    Iron::new(chain).http(&*server_addr).unwrap();
}

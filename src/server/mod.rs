use iron::prelude::*;
use logger::Logger;
use mount::Mount;

use config::CONFIG;

mod handlers;

// TODO logging middleware

pub fn serve() {
    let mut mount = Mount::new();

    mount.mount("/summary/",
                router!(
        get "/" => handlers::summary
    ));

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

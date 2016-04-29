use iron::prelude::*;
use mount::Mount;

use config::CONFIG;

mod handlers;

// TODO logging middleware

pub fn serve() {
    let mut mount = Mount::new();

    mount.mount("/summary/",
                router!(
        get "/" => handlers::default_summary,
        get "/:start/:end" => handlers::summary,
    ));

    let server_addr = format!("0.0.0.0:{}", CONFIG.server_port);

    info!("API server running at {}", &server_addr);
    Iron::new(mount).http(&*server_addr).unwrap();
}

use iron::prelude::*;
use iron::status;
use router::Router;
use serde_json::ser;

use error::DashError;
use nag;

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

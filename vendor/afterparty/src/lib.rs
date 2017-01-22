//! Afterparty is a github webhook handler library for building custom integrations

#[macro_use]
extern crate log;
#[macro_use]
extern crate hyper;
extern crate case;
extern crate crypto;
extern crate iron;
extern crate serde;
extern crate serde_json;
extern crate hex;

mod hook;
mod events;

pub use events::Event;
pub use hook::{AuthenticateHook, Hook};
use std::collections::HashMap;
use std::io::Read;

/// signature for request
/// see [this document](https://developer.github.com/webhooks/securing/) for more information
header! {(XHubSignature, "X-Hub-Signature") => [String]}

/// name of Github event
/// see [this document](https://developer.github.com/webhooks/#events) for available types
header! {(XGithubEvent, "X-Github-Event") => [String]}

/// unique id for each delivery
header! {(XGithubDelivery, "X-Github-Delivery") => [String]}

/// A delivery encodes all information about web hook request
#[derive(Debug)]
pub struct Delivery<'a> {
    pub id: &'a str,
    pub event: &'a str,
    pub payload: Event,
    pub unparsed_payload: &'a str,
    pub signature: Option<&'a str>,
}

impl<'a> Delivery<'a> {
    pub fn new(id: &'a str,
               event: &'a str,
               payload: &'a str,
               signature: Option<&'a str>)
               -> Option<Delivery<'a>> {

        // patching raw payload with camelized name field for enum deserialization
        let patched = events::patch_payload_json(event, payload);
        match serde_json::from_str::<Event>(&patched) {
            Ok(parsed) => {
                Some(Delivery {
                    id: id,
                    event: event,
                    payload: parsed,
                    unparsed_payload: payload,
                    signature: signature,
                })
            }
            Err(e) => {
                error!("failed to parse json {:?}\n{:#?}", e, patched);
                None
            }
        }
    }
}

/// A hub is a registry of hooks
#[derive(Default)]
pub struct Hub {
    hooks: HashMap<String, Vec<Box<Hook>>>,
}

impl Hub {
    /// construct a new hub instance
    pub fn new() -> Hub {
        Hub { ..Default::default() }
    }

    /// adds a new web hook which will only be applied
    /// when a delivery is revcieved with a valid
    /// request signature based on the provided secret
    pub fn handle_authenticated<H, S>(&mut self, event: &str, secret: S, hook: H)
        where H: Hook + 'static,
              S: Into<String>
    {
        self.handle(event, AuthenticateHook::new(secret, hook))
    }

    /// add a need hook to list of hooks
    /// interested in a given event
    pub fn handle<H>(&mut self, event: &str, hook: H)
        where H: Hook + 'static
    {
        self.hooks
            .entry(event.to_owned())
            .or_insert(vec![])
            .push(Box::new(hook));
    }

    /// get all interested hooks for a given event
    pub fn hooks(&self, event: &str) -> Option<Vec<&Box<Hook>>> {
        let explicit = self.hooks.get(event);
        let implicit = self.hooks.get("*");
        let combined = match (explicit, implicit) {
            (Some(ex), Some(im)) => {
                Some(ex.iter().chain(im.iter()).into_iter().collect::<Vec<_>>())
            }
            (Some(ex), _) => Some(ex.into_iter().collect::<Vec<_>>()),
            (_, Some(im)) => Some(im.into_iter().collect::<Vec<_>>()),
            _ => None,
        };
        combined
    }
}

impl iron::middleware::Handler for Hub {
    fn handle(&self, req: &mut iron::Request) -> iron::IronResult<iron::Response> {
        let headers = req.headers.clone();
        if let (Some(&XGithubEvent(ref event)), Some(&XGithubDelivery(ref delivery))) =
               (headers.get::<XGithubEvent>(), headers.get::<XGithubDelivery>()) {
            let signature = headers.get::<XHubSignature>();
            info!("recv '{}' event with signature '{:?}'", event, signature);
            if let Some(hooks) = self.hooks(event) {
                let mut payload = String::new();
                if let Ok(_) = req.body.read_to_string(&mut payload) {
                    match Delivery::new(delivery,
                                        event,
                                        payload.as_ref(),
                                        signature.map(|s| s.as_ref())) {
                        Some(delivery) => {
                            for hook in hooks {
                                hook.handle(&delivery);
                            }
                        }
                        _ => {
                            error!("failed to parse event {:?} for delivery {:?}",
                                   event,
                                   delivery)
                        }
                    }
                }
            }
        }
        Ok(iron::Response::with((iron::status::Ok, "ok")))
    }
}

#[cfg(test)]
mod tests {
    use super::{Delivery, Hub};

    #[test]
    fn hub_hooks() {
        let mut hub = Hub::new();
        // UFCS may be required is hyper::server::Handler is in scope
        // Hub::handle(&mut hub, "push", |_: &Delivery| {});
        // Hub::handle(&mut hub, "*", |_: &Delivery| {});
        hub.handle("push", |_: &Delivery| {});
        hub.handle("*", |_: &Delivery| {});
        assert_eq!(Some(2),
                   hub.hooks("push").map(|hooks| hooks.into_iter().count()))
    }
}

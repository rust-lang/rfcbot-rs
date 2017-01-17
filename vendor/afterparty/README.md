# afterparty

[![Build Status](https://travis-ci.org/softprops/afterparty.svg?branch=master)](https://travis-ci.org/softprops/afterparty) [![Coverage Status](https://coveralls.io/repos/github/softprops/afterparty/badge.svg?branch=master)](https://coveralls.io/github/softprops/afterparty?branch=master) [![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg)](LICENSE) [![crates.io](http://meritbadge.herokuapp.com/afterparty)](https://crates.io/crates/afterparty)

> Where your commits go after github

Afterparty is a library for building Github webhook integrations in Rust.

## docs

Find them [here](http://softprops.github.io/afterparty)

## install

Add the following to your `Cargo.toml` file

```toml
[dependencies]
afterparty = "0.1"
```

## usage

Afterparty has two key abstractions, a `Hook`: a handler interface webhook deliveries, and a `Hub`: a registry for hooks. A `Hub` provides `Delivery` instances to interested hooks.

A `Delivery` encodes all relevant webhook request information including a unique identifier for the delivery, the event name, and statically typed payload represented as an enumerated type of `Event`.

Hooks subscribe to [Events](https://developer.github.com/webhooks/#events) via `Hub`'s a `handle` and `handle_authenticated` functions.
To subscribe to multiple events, subscribe with "*" and pattern match on the provided delivery's payload value.

To register your webhook with Github visit your repo's hooks configuration form `https://github.com/{login}/{repo}/settings/hooks/new` and select the events you
want Github to notify your server about.

Hubs implements [Hyper](https://github.com/hyperium/hyper)'s Server Handler trait so that it may be mounted into any hyper Server.

```rust
extern crate afterparty;
extern crate hyper;

use hyper::Server;
use afterparty::{Delivery, Event, Hub};

fn main() {
    let mut hub = Hub::new();
    hub.handle("*", |delivery: &Delivery| {
        println!("rec delivery {:#?}", delivery)
    });
    hub.handle_authenticated("pull_request", "secret", |delivery: &Delivery| {
       println!("rec authenticated delivery");
       match delivery.payload {
           Event::PullRequest { ref action, ref sender, .. } => {
               println!("sender {} action {}", sender.login, action)
           },
           _ => ()
       }
    });
    let svc = Server::http("0.0.0.0:4567")
       .unwrap()
       .handle(hub);
    println!("hub is up");
    svc.unwrap();
}
```

### note on UFCS

In the case that you have hyper::server::Handler and hubcaps::Hub in scope you may need to use UFCS to invoke
the handle method on a HUB instance.

For example...

```rust
extern crate afterparty;
extern crate hyper;

use hyper::server::Handle;
use afterparty::{Delivery, Hub};

fn main() {
    let mut hub = Hub::new();
    hubcaps::Hub::handle(&mut hub, "*", |delivery: &Delivery| { });
}
```

## building

As far as rust project builds go this one is somewhat interesting. This library uses serde for json encoding/decoding
and is focused on stable rust releases so a tactic for code generatation at build time is employed. Before that happens
an attempt is made to synthesize structs based on Github api documentation json vendored in the data directory.
A known issue exists where the repo `deployments_url` field is omitted with a fresh set of json. serde will error at
deserializing because of this. This field was hand added within the json vendored dataset for the time being. Serde 0.7
will likely be released soon and will enable this library to avoid these kinds of runtime deserialization errors for
missing fields.

Doug Tangren (softprops) 2015-2016

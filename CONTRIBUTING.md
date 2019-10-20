# rfcbot-rs

[![Travis](https://img.shields.io/travis/rust-lang/rust.svg)](https://travis-ci.com/rust-lang/rfcbot-rs)

Deployed to https://rfcbot.rs right now.

## Code of Conduct

All contributors are expected to follow our [Code of Conduct][conduct].

## Development

### Chat

There is an `#rfcbot` channel in the `ops` section of the 
[rust-lang discord server](https://discordapp.com/invite/rust-lang).

### Rust Version

Rust nightly is required, as rfcbot uses [Rocket](rocket.rs). If you use rustup, this version is 
managed for you by a `rust-toolchain` file at the repository root.

### Running locally

Install [Docker](https://docker.dom) and [docker-compose](https://docs.docker.com/compose) and 
ensure they're working.

Next, start development services and initialize the database with `docker-compose up`.

Press `Ctrl+C` to exit the server process. The container build is not (yet?) aware of how to detect
changes to source files, so once you have made changes you will need to run `docker-compose build`
before running `docker-compose up` to see your changes take effect.

By default this stores your database files in `target/data/`, so any temporary changes you make to 
the database will be removed by a `cargo clean` and you'll need to run the above commands again.

### Database dumps

It can be useful to have a database with some existing data to start from. "Bootstrap" files are 
available at https://www.dropbox.com/sh/dl4pxj1d49ici1f/AAAzZQxWVqQzVk_zOksn0Rbya?dl=0.

Assuming that you download the most recent file in the above folder and name it `bootstrap.sql`:

```bash
# see setup-db.sh for the url to use here
psql -d $DATABASE_URL -f bootstrap.sql
```

## Deployment

Deployed to Heroku via TravisCI from the master branch. See [.travis.yml](./travis.yml) for an 
up-to-date listing of the actions.

## License

This project is distributed under the terms of both the MIT license and the Apache License 
(Version 2.0). See LICENSE-MIT and LICENSE-APACHE for details.

[conduct]: https://www.rust-lang.org/conduct.html
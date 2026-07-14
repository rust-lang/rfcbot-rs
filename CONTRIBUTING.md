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

### Without Docker

### Configure a database

To use Postgres, you will need to install it and configure it:

1. Install Postgres. Look online for any help with installing and setting up Postgres (particularly if you need to create a user and set up permissions).
2. Login into Postgres (optionally setting host and port)
   ```
   sudo su - postgres
   psql -d template1 -U postgres
   ```
2. Create a DB user: `CREATE ROLE rfcbot LOGIN PASSWORD 'pass';`
3. Create the rfcbot DB: `CREATE DATABASE rfcbot with ENCODING 'UTF8' LC_COLLATE='C' LC_CTYPE='C' template=template0 owner=rfcbot;`
4. Login is now possible with the new user: `psql rfcbot -U rfcbot -W`
4. In the `.env` file, set the `DATABASE_URL`:
   ```sh
   DATABASE_URL=postgres://rfcbot:pass@localhost/rfcbot
   ```

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


[conduct]: https://rust-lang.org/policies/code-of-conduct

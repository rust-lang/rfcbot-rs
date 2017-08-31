# rust-dashboard

[![Travis](https://img.shields.io/travis/rust-lang/rust.svg)](https://travis-ci.org/dikaiosune/rust-dashboard)

Deployed to http://rusty-dash.com right now.

## ToC

* [Development](#development)
  * [Configuring environment variables](#configuring-environment-variables)
  * [Running server processes](#running-server-processes)
  * [Database Connection](#database-connection)
* [Configuration](#configuration)
  * [Rust Version](#rust-version)
  * [Environment variables](#environment-variables)
* [Database](#database)
* [Bootstrapping](#bootstrapping)
* [Scraping](#scraping)
* [Deployment](#deployment)
* [License](#license)

## Development

A development environment is available using Vagrant. [Install Vagrant](https://www.vagrantup.com/docs/installation/) along with a VM provider (tested with VirtualBox). It's a good idea to install the [vagrant-vbguest](https://github.com/dotless-de/vagrant-vbguest) plugin, as well. Once Vagrant and a VM provider are installed, you'll need to configure a couple of environment variables, (potentially) run DB migrations, and start the server processes. If you run into any issues provisioning the development environment, please file an issue!

### Configuring environment variables

Most of the configuration has some default set (see [vagrant_env.sh](https://github.com/dikaiosune/rust-dashboard/blob/master/vagrant_env.sh)), but you'll need to configure access to the GitHub API for testing the scraper. Something like this in the root project directory should suffice:

```
$ touch .env
$ echo "GITHUB_ACCESS_TOKEN=your_github_access_token_see_config_section_for_details" >> .env
$ echo "GITHUB_USER_AGENT=your_github_username" >> .env
$ echo "POST_COMMENTS=false" >> .env
```

**NOTE:** While the dashboard doesn't require any permissions boxes to be checked in access token creation, and the code makes every effort to avoid modifying any state through GitHub's API, there's always a risk with handing 3rd-party code your API credentials.

### Database dumps

It can be useful to have a database with some existing data to start from. "Bootstrap" files are available at https://www.dropbox.com/sh/dl4pxj1d49ici1f/AAAzZQxWVqQzVk_zOksn0Rbya?dl=0. They usually are behind several migrations, so you'll still need to run the migrations if you start from one.

### Running server processes

There are two daemons to run, one for the front-end development server, and one for the back-end API server and scraper. It's recommended to run these in two separate terminal windows/tabs/sessions.

You may need to run database migrations if the bootstrap SQL file is stale:

```
$ diesel migration run
```

To run the back-end API server and scraper:

```
$ cargo run
```

**NOTE:** The API server process needs to be manually restarted whenever you want to see code changes reflected in their behavior, or whenever you run migrations on the test database. A `Ctrl+C` followed by `Up` and `Enter` usually works if running them through cargo. `cargo watch` is also a nice tool.

### Database connection

If you want to perform any database action, make sure you have a reachable installation of PostgreSQL that is configured with the DATABASE_URL environment variable.

## Configuration

### Rust Version

Rust nightly is required, as rfcbot uses [Rocket](rocket.rs) now. Pin `rustup` to the correct version:

```
$ rustup override set nightly-2017-08-26
```

### Environment variables

Note that you can configure the Rocket web server using environment variables like `ROCKET_PORT`, according to the Rocket [configuration guide](https://rocket.rs/guide/configuration/).

* `DATABASE_URL`: postgres database URL
* `DATABASE_POOL_SIZE`: number of connections to maintain in the pool
* `GITHUB_ACCESS_TOKEN`: your access token from GitHub. See [this page](https://help.github.com/articles/creating-an-access-token-for-command-line-use/) for more information. You shouldn't need to check any of the boxes for granting scopes when creating it.
* `GITHUB_USER_AGENT`: the UA string to send to GitHub (they request that you send your GitHub username or the app name you registered for the client ID)
* `GITHUB_WEBHOOK_SECRETS`: a comma-delimited string of the secrets used for any ingestion webhooks. The webhook handler will attempt to validate any POST'd webhook against each secret until it either finds a matching one or runs out.
* `RUST_LOG`: the logging configuration for [env_logger](https://crates.io/crates/env_logger). If you're unfamiliar, you can read about it in the documentation linked on crates.io. If it's not defined, logging will default to `info!()` and above.
* `GITHUB_SCRAPE_INTERVAL`: time (in minutes) to wait in between GitHub scrapes
* `POST_COMMENTS`: whether to post RFC bot comments on issues -- either `true` or `false`. Be very careful setting to true when testing -- it will post comments using whatever account is associated with the GitHub API key you provide.

## Database

I'm testing with PostgreSQL 9.5. To init, make sure `DATABASE_URL` is set, and:

```
cargo install diesel_cli
diesel migration run
diesel print-schema > src/domain/schema.rs
```

That should have the database you've specified ready to receive data. Then you can run some of the bootstrapping commands (see below). Alternatively, you can use `bootstrap.sql` to get a nice starting point for the database (note that this isn't maintained regularly).

```bash
psql -d $DB_NAME_HERE -f bootstrap.sql
```

## Deployment

Setup a postgres database and user on a server with `dpkg` (recent Ubuntu is what's tested), and install nginx.

Run `build.sh` on that machine. This will create a `rust-dashboard.deb` in the repo root. Install with `dpkg -i rust-dashboard.deb`, configure `/etc/rust-dashboard/env` from the example file there, and start the services:

```bash
sudo systemctl enable rust-dashboard-api
sudo systemctl start rust-dashboard-api
```

Hopefully, that'll all *just work*. Haha.

## License

This project is distributed under the terms of both the MIT license and the Apache License (Version 2.0). See LICENSE-MIT and LICENSE-APACHE for details.

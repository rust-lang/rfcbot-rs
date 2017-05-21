# rust-dashboard

[![Travis](https://img.shields.io/travis/rust-lang/rust.svg)](https://travis-ci.org/dikaiosune/rust-dashboard)

Deployed to http://rusty-dash.com right now.

## ToC

* [Development](#development)
  * [Configuring environment variables](#configuring-environment-variables)
  * [Running server processes in Vagrant](#running-server-processes-in-vagrant)
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

### Running server processes in Vagrant

There are three daemons to run, one each for the front-end development server, the back-end API server, and the scraper. It's recommended to run these in three separate terminal windows/tabs/sessions. Assuming the VM is already running (`vagrant up`), you'll need to run `vagrant ssh` in each terminal session to access the VM.

You may need to run database migrations if the bootstrap SQL file is stale:

```
$ cd /vagrant && diesel migration run
```

To run the back-end API server:

```
$ cd /vagrant && cargo run -- serve
```

To run the scraper daemon:

```
$ cd /vagrant && cargo run -- scrape
```

**NOTE:** The API server and scraper processes need to be manually restarted whenever you want to see code changes reflected in their behavior, or whenever you run migrations on the test database. A `Ctrl+C` followed by `Up` and `Enter` usually works if running them through cargo.

To install dependencies for the front-end development server and run it:

```
$ cd /vagrant/front
$ ember server --proxy=http://localhost:8080
```

You can then browse (on your host machine) to `http://localhost:4040` to view the development server output.

### Database Connection

Assuming that the VM provisions correctly (a little bit of an "if"), you should be able to connect to the PostgreSQL database on the host machine's port `4050`, using user: `vagrant` and password: `hunter2`.

## Configuration

### Rust Version

Rust 1.17 or later is required.

### Environment variables

* `DATABASE_URL`: postgres database URL
* `DATABASE_POOL_SIZE`: number of connections to maintain in the pool
* `GITHUB_ACCESS_TOKEN`: your access token from GitHub. See [this page](https://help.github.com/articles/creating-an-access-token-for-command-line-use/) for more information. You shouldn't need to check any of the boxes for granting scopes when creating it.
* `GITHUB_USER_AGENT`: the UA string to send to GitHub (they request that you send your GitHub username or the app name you registered for the client ID)
* `GITHUB_WEBHOOK_SECRETS`: a comma-delimited string of the secrets used for any ingestion webhooks. The webhook handler will attempt to validate any POST'd webhook against each secret until it either finds a matching one or runs out.
* `RUST_LOG`: the logging configuration for [env_logger](https://crates.io/crates/env_logger). If you're unfamiliar, you can read about it in the documentation linked on crates.io. If it's not defined, logging will default to `info!()` and above.
* `GITHUB_SCRAPE_INTERVAL`: time (in minutes) to wait in between GitHub scrapes
* `RELEASES_SCRAPE_INTERVAL`: time (in minutes) to wait in between nightly release scrapes
* `BUILDBOT_SCRAPE_INTERVAL`: time (in minutes) to wait in between buildbot scrapes
* `SERVER_PORT`: port on which the API server should listen
* `POST_COMMENTS`: whether to post RFC bot comments on issues -- either `true` or `false`. Be very careful setting to true when testing -- it will post comments using whatever account is associated with the GitHub API key you provide.

## Database

I'm testing with PostgreSQL 9.5. To init, make sure `DATABASE_URL` is set, and:

```
cargo install diesel_cli
diesel migration run
```

That should have the database you've specified ready to receive data. Then you can run some of the bootstrapping commands (see below). Alternatively, you can use `bootstrap.sql` to get a nice starting point for the database (note that this isn't maintained regularly).

```bash
psql -d $DB_NAME_HERE -f bootstrap.sql
```

## Bootstrapping

Run `cargo run --release -- bootstrap SOURCE YYYY-MM-DD` to populate the database with all relevant data since YYYY-MM-DD.

Substitute `SOURCE` in that example with one of:

* `github`
* `releases`
* `buildbot`

The date can also be replaced by `all`, which will scrape all data available since 2015-05-15 (Rust's 1.0 launch). Any date will be ignored for the buildbot command, as the API doesn't support queries for recently updated items.

## Scraping

The launch the scraping daemon, make sure the interval environment variables are set to sensible values, and run `cargo run --release -- scrape`. This will scrape each data source, sleeping each data source's scraper for the given interval in between runs. Some important notes about the current setup:

* **IMPORTANT**: if the scraper daemon is killed in the middle of scraping and persisting a data source, it's a *very* good idea to run the bootstrap command for that/those data source(s). Current the database schema doesn't house any check-pointing or machine-readable confirmation of a successful scraping, which could result in erroneous holes in data for APIs which support "all results updated since TIME" queries (like GitHub). This is due to the fact that the current scraper just checks for the most recent entities in each category before telling the API how far back it wants to go.
* In order to avoid overloading services, make sure that the intervals are not too small. Some examples:
  * The GitHub API allows (at the time of this writing) 5000 authenticated requests per hour. The GitHub scraper currently makes 1 request for every 100 updated issues, 1 request for every 100 update issue comments, and **1 request for every updated pull request**. Granted, this API limit is only likely to be an issue when bootstrapping the entire repository history, but bear it in mind if setting very low intervals (e.g. 1-2 minutes) or if using the same GitHub account for multiple API-using tools.
  * The buildbot scraper takes 5-20 minutes as of this writing, and could potentially place significant load on the CI cluster since it requests all build records which may have to be deserialized from disk. Make sure to space this scraper out accordingly so that it is not running a significant percentage of the time.

## Deployment

Setup a postgres database and user on a server with `dpkg` (recent Ubuntu is what's tested), and install nginx.

Run `build.sh` on that with a compatible nightly installed (2016-10-18 right now). This will create a `rust-dashboard.deb` in the repo root. Install with `dpkg -i rust-dashboard.deb`, configure `/etc/rust-dashboard/env` from the example file there, and start the services:

```bash
sudo systemctl enable rust-dashboard-scraper
sudo systemctl enable rust-dashboard-api
sudo systemctl start rust-dashboard-scraper
sudo systemctl start rust-dashboard-api
```

Hopefully, that'll all *just work*. Haha.

## License

This project is distributed under the terms of both the MIT license and the Apache License (Version 2.0). See LICENSE-MIT and LICENSE-APACHE for details.

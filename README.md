# rust-dashboard

Nothing to see here yet. Move along.

## Configuration

### Rust Version

Not guaranteed to be updated regularly, but last time I remembered to update this, the dashboard compiled with:

```bash
$ rustc --version
rustc 1.10.0-nightly (c2aaad4e2 2016-04-19)
```

### Environment variables

* `DATABASE_URL`: postgres database URL
* `DATABASE_POOL_SIZE`: number of connections to maintain in the pool
* `GITHUB_ACCESS_TOKEN`: your access token from GitHub. See [this page](https://help.github.com/articles/creating-an-access-token-for-command-line-use/) for more information. You shouldn't need to check any of the boxes for granting scopes when creating it.
* `GITHUB_USER_AGENT`: the UA string to send to GitHub (they request that you send your GitHub username or the app name you registered for the client ID)
* `RUST_LOG`: the logging configuration for [env_logger](https://crates.io/crates/env_logger). If you're unfamiliar, you can read about it in the documentation linked on crates.io. If it's not defined, logging will default to `info!()` and above.

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

This doesn't *fully* work yet, but run `cargo run --release -- bootstrap SOURCE YYYY-MM-DD` to populate the database with all relevant data since YYYY-MM-DD.

Substitute `SOURCE` in that example with one of:

* `github`
* `releases`
* `buildbot`

The date can also be replaced by `all`, which will scrape all data available since 2015-05-15 (Rust's 1.0 launch).

## License

This project is distributed under the terms of both the MIT license and the Apache License (Version 2.0). See LICENSE-MIT and LICENSE-APACHE for details.

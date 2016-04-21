# rust-dashboard

Nothing to see here yet. Move along.

## Configuration

### Environment variables

* `DATABASE_URL`: postgres database URL
* `DATABASE_POOL_SIZE`: number of connections to maintain in the pool
* `GITHUB_ACCESS_TOKEN`: your access token from GitHub. See [this page](https://help.github.com/articles/creating-an-access-token-for-command-line-use/) for more information. You shouldn't need to check any of the boxes for granting scopes when creating it.
* `GITHUB_USER_AGENT`: the UA string to send to GitHub (they request that you send your GitHub username or the app name you registered for the client ID)

## Database

I'm testing with PostgreSQL 9.5. To init, make sure `DATABASE_URL` is set, and:

```
cargo install diesel_cli
diesel migration run
```

That should then have whichever database you've specified ready to receive data.

## Bootstrapping

This doesn't *fully* work yet, but run `cargo run --release -- bootstrap SOURCE YYYY-MM-DD` to populate the database with all relevant data since YYYY-MM-DD.

Substitute `SOURCE` in that example with one of:

* `github`
* `release-channel`

## License

This project is distributed under the terms of both the MIT license and the Apache License (Version 2.0). See LICENSE-MIT and LICENSE-APACHE for details.

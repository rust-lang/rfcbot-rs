# CHANGELOG

`@rfcbot` is Rust's automated process manager for RFCs, tracking issues, etc.
As such, this application does not follow semver of any form.

However, we do maintain this changelog explaining high level changes to the way
rfcbot behaves so that the people who develop rfcbot or interact with it can
understand the bot better.

## Changes

+ The bot will now optionally (per configuration in `rfcbot.toml`) remove
  unwanted reactions from RFC (and PR..) readers.
  These include 👎, 😕 on the `rust-lang/rfcs` and `rust-lang/rust` repositories.

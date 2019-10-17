###############################################################################
# builder image

FROM clux/muslrust:stable as builder

WORKDIR /rfcbot
RUN USER=root cargo init --vcs none

COPY rust-toolchain ./
RUN rustc --version && rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

COPY . ./
# cargo apparently uses mtime and docker doesn't modify it, needed to rebuild:
RUN touch src/main.rs
RUN cargo build --release --locked

###############################################################################
# runner image

FROM alpine:latest
RUN apk --no-cache add ca-certificates

# heroku runs as non-root
RUN adduser -D notroot
USER notroot

COPY --from=builder /rfcbot/target/x86_64-unknown-linux-musl/release/rfcbot-rs /usr/local/bin/rfcbot
CMD ROCKET_PORT=$PORT /usr/local/bin/rfcbot

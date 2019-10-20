###############################################################################
# builder image

FROM clux/muslrust:stable as builder

# postgres client is used to gate test server start, diesel_cli runs test migrations and init
RUN apt-get update && apt-get install -y libpq-dev openssl pkg-config postgresql-client
RUN rustup target add --toolchain stable x86_64-unknown-linux-gnu
RUN cargo +stable install --target x86_64-unknown-linux-gnu \
    diesel_cli --no-default-features --features postgres
ENV PATH="${PATH}:${HOME}/.cargo/bin"

WORKDIR /rfcbot
RUN USER=root cargo init --vcs none

COPY rust-toolchain ./
RUN rustc --version && rustup target add x86_64-unknown-linux-musl

COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --locked

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

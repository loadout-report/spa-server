# Build Stage
FROM rust:latest AS builder

RUN rustup default nightly
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /usr/src/
# RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/spa-server
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Bundle Stage
FROM gcr.io/distroless/static-debian11:nonroot
COPY --from=builder /usr/src/loadout.report/target/x86_64-unknown-linux-musl/release/spa-server /
CMD ["/spa-server"]
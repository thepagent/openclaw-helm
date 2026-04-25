FROM rust:1.77-alpine AS builder
RUN apk add --no-cache musl-dev pkgconfig openssl-dev
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM alpine:3.19
RUN apk add --no-cache ca-certificates
COPY --from=builder /build/target/release/gatekeeper /usr/local/bin/gatekeeper
ENTRYPOINT ["/usr/local/bin/gatekeeper"]

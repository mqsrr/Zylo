ARG RUST_VERSION=1.85.0-bookworm
FROM rust:${RUST_VERSION} AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler libprotobuf-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /usr/src/app/target/release/aggregator /usr/local/bin/aggregator-service

EXPOSE 8080

CMD ["/usr/local/bin/aggregator-service"]
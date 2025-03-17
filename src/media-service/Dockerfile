ARG RUST_VERSION=1.85.0-bookworm
FROM rust:${RUST_VERSION} AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/media-service

COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /usr/src/media-service/target/release/media-service /usr/local/bin/media-service

EXPOSE 8080
EXPOSE 50051

CMD ["/usr/local/bin/media-service"]
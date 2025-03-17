ARG RUST_VERSION=1.85.0-bookworm
FROM rust:${RUST_VERSION} AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/user-interaction

COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /usr/src/user-interaction/target/release/user-interaction /usr/local/bin/user-interaction

EXPOSE 8080
EXPOSE 50051

CMD ["/usr/local/bin/user-interaction"]

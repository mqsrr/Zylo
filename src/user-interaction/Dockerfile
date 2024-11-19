ARG RUST_VERSION=1.82.0-bullseye
ARG APP_NAME=user-interaction

FROM rust:${RUST_VERSION} AS builder
ARG APP_NAME

WORKDIR /usr/src/${APP_NAME}

COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
ARG APP_NAME
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/${APP_NAME} /usr/local/bin/${APP_NAME}

EXPOSE 8080
CMD ["${APP_NAME}"]
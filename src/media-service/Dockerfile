# syntax=docker/dockerfile:1

# Use an argument for the Rust version for flexibility
ARG RUST_VERSION=1.82.0-bullseye

# Build stage
FROM rust:${RUST_VERSION} AS builder

# Install required build tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/src/media-service

# Copy the source code
COPY . .

# Install dependencies and build the application
RUN cargo build --release

# Production stage
FROM debian:bullseye-slim

# Install required runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/media-service/target/release/media-service /usr/local/bin/media-service

# Expose the application port
EXPOSE 8080

# Command to run the application
CMD ["/usr/local/bin/media-service"]

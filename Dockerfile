# syntax=docker/dockerfile:1.4

########## Builder Stage ##########
FROM rust:1.88-bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /hooksmith

# Copy only essential files for building
COPY Cargo.toml Cargo.lock* ./
COPY src/ ./src/
COPY crates/ ./crates/

# Build only the main Hooksmith binary and entrypoint
RUN cargo build --release --bin hooksmith --bin docker-entrypoint

########## Runtime Stage ##########
FROM debian:bookworm-slim

# Install minimal dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=builder /hooksmith/target/release/hooksmith /usr/local/bin/hooksmith
COPY --from=builder /hooksmith/target/release/docker-entrypoint /usr/local/bin/docker-entrypoint

# Use exec form for clean argument passing
ENTRYPOINT ["/usr/local/bin/docker-entrypoint"]
CMD ["hooksmith", "--help"]

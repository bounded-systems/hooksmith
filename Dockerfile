# syntax=docker/dockerfile:1.4

########## Rust Base Stage ##########
FROM rust:1.88-bookworm as rust-base

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /hooksmith

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock* ./

# Fetch dependencies (cached layer)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo fetch

########## Development Stage ##########
FROM rust-base as dev

# Copy source code
COPY . .

# Build with cache mounts for maximum performance
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/hooksmith/target \
    cargo build --release --bin hooksmith --bin docker-entrypoint --bin hooksmith-tasks

########## Test Stage ##########
FROM dev as test

# Run tests with cache mounts
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/hooksmith/target \
    cargo test --release

########## Production Stage ##########
FROM debian:bookworm-slim as hooksmith

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binaries from dev stage
COPY --from=dev /hooksmith/target/release/hooksmith /usr/local/bin/hooksmith
COPY --from=dev /hooksmith/target/release/docker-entrypoint /usr/local/bin/docker-entrypoint

# Use exec form for clean argument passing
ENTRYPOINT ["/usr/local/bin/docker-entrypoint"]
CMD ["hooksmith", "--help"]

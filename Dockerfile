# syntax=docker/dockerfile:1.4

########## Builder Stage ##########
FROM rust:1.77-slim as builder

WORKDIR /hooksmith

# Copy full source code
COPY . .

# Build Hooksmith binaries
RUN cargo build --release

########## Runtime Stage ##########
FROM debian:bullseye-slim

# Install minimal dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /hooksmith/target/release/hooksmith /usr/local/bin/hooksmith

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Use exec form for clean argument passing
ENTRYPOINT ["/entrypoint.sh"]
CMD ["hooksmith", "--help"]

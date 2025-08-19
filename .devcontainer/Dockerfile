# Multi-stage Dockerfile for secure development environment
# Based on Nix for reproducible builds

# Stage 1: Base Nix environment
FROM ghcr.io/cachix/devenv:latest AS base

# Security: Update base system and install security tools
USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    gnupg \
    software-properties-common \
    && rm -rf /var/lib/apt/lists/*

# Stage 2: Security tools installation  
FROM base AS security
USER root

# Install security scanning tools
RUN curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin \
    && curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sh -s -- -b /usr/local/bin

# Install Cosign for artifact signing
RUN curl -O -L "https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64" \
    && mv cosign-linux-amd64 /usr/local/bin/cosign \
    && chmod +x /usr/local/bin/cosign

# Install additional security tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    git-secrets \
    trivy \
    && rm -rf /var/lib/apt/lists/*

# Stage 3: Development environment
FROM security AS development

# Create non-root development user with proper permissions
RUN groupadd --gid 1000 devenv \
    && useradd --uid 1000 --gid devenv --shell /bin/bash --create-home devenv \
    && usermod -aG sudo devenv \
    && echo 'devenv ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers.d/devenv \
    && chmod 0440 /etc/sudoers.d/devenv

# Set up secure directories with proper permissions
RUN mkdir -p /workspace /home/devenv/.config /home/devenv/.cache \
    && chown -R devenv:devenv /workspace /home/devenv

# Switch to non-root user for security
USER devenv
WORKDIR /workspace

# Configure Nix for the user
RUN mkdir -p /home/devenv/.config/nix \
    && echo "experimental-features = nix-command flakes" > /home/devenv/.config/nix/nix.conf \
    && echo "substituters = https://cache.nixos.org/ https://hooksmith.cachix.org" >> /home/devenv/.config/nix/nix.conf \
    && echo "trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=" >> /home/devenv/.config/nix/nix.conf

# Set environment variables for security and reproducibility
ENV SOURCE_DATE_EPOCH=1
ENV RUSTFLAGS="-D warnings"
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

# Health check for container
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD nix --version && test -d /workspace

# Stage 4: Production-ready distroless (for actual releases)
FROM gcr.io/distroless/cc-debian12:nonroot AS production

# Copy only necessary binaries from development stage
COPY --from=development --chown=nonroot:nonroot /nix/store /nix/store
COPY --from=development --chown=nonroot:nonroot /usr/local/bin/syft /usr/local/bin/grype /usr/local/bin/cosign /usr/local/bin/

# Use non-root user
USER nonroot:nonroot
WORKDIR /workspace

# Minimal entry point
ENTRYPOINT ["/bin/sh"]

# Default target is development for DevContainer
FROM development AS final

# Security: Scan the final image (can be done in CI)
# RUN grype dir:/ --scope all-layers -o table || true

# Final security hardening
USER devenv
WORKDIR /workspaces/hooksmith

# Set secure defaults
ENV DEBIAN_FRONTEND=noninteractive
ENV TERM=xterm-256color
ENV SHELL=/bin/bash

# Document exposed ports (for reference)
EXPOSE 3000 8080

# Final validation
RUN nix --version \
    && syft --version \
    && grype version \
    && cosign version

# Development-friendly entry point
CMD ["/bin/bash"]

# Multi-stage build for AgentGraph
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build dependencies (this is cached unless Cargo.toml changes)
RUN cargo build --release --workspace

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --workspace

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false agentgraph

# Set working directory
WORKDIR /app

# Copy built binaries
COPY --from=builder /app/target/release/agent-graph* /usr/local/bin/

# Copy configuration files
COPY --from=builder /app/examples/ ./examples/

# Change ownership
RUN chown -R agentgraph:agentgraph /app

# Switch to app user
USER agentgraph

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /usr/local/bin/agent-graph --version || exit 1

# Default command
CMD ["agent-graph"]

# Metadata
LABEL org.opencontainers.image.title="AgentGraph"
LABEL org.opencontainers.image.description="Production-grade multi-agent framework for Rust"
LABEL org.opencontainers.image.version="0.4.0"
LABEL org.opencontainers.image.authors="AgentGraph Contributors"
LABEL org.opencontainers.image.source="https://github.com/agent-graph/agent-graph"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
# ============================================================================
# Ledger Service — Multi-stage Docker Build
# ============================================================================
# Stage 1: Build the Rust application
# Stage 2: Create a minimal runtime image
#
# This is similar to a Maven multi-stage build in Java:
#   Stage 1 = mvn package (compile + dependencies)
#   Stage 2 = Copy the JAR into a slim JRE image
#
# In Rust, the compiled binary is a single static executable — no JVM needed.
# ============================================================================

# ---------------------------------------------------------------------------
# Stage 1: Builder
# ---------------------------------------------------------------------------
FROM rust:1.83-bookworm AS builder

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies (cache layer)
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source code
COPY src/ src/
COPY migrations/ migrations/

# Touch main.rs to invalidate the cached dummy build
RUN touch src/main.rs

# Build the real application
RUN cargo build --release

# ---------------------------------------------------------------------------
# Stage 2: Runtime
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies (OpenSSL, CA certificates)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/ledger-service /app/ledger-service

# Copy migrations for runtime migration execution
COPY --from=builder /app/migrations /app/migrations

# Create a non-root user for security
RUN groupadd -r ledger && useradd -r -g ledger ledger
USER ledger

# Expose the application port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
ENTRYPOINT ["/app/ledger-service"]

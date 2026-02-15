# --- Builder ---
FROM rust:1.93-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependencies: copy manifests first, build a dummy to populate cargo cache
COPY Cargo.toml Cargo.lock* ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY shared/Cargo.toml shared/Cargo.toml
COPY cli/Cargo.toml cli/Cargo.toml

RUN mkdir -p backend/src shared/src cli/src \
    && echo "fn main() {}" > backend/src/main.rs \
    && echo "" > shared/src/lib.rs \
    && echo "fn main() {}" > cli/src/main.rs \
    && cargo build --release --bin axum-api 2>/dev/null || true \
    && rm -rf backend/src shared/src cli/src

# Build the real binary
COPY shared/ shared/
COPY backend/ backend/

RUN cargo build --release --bin axum-api

# --- Runtime ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/axum-api /usr/local/bin/axum-api

EXPOSE 3069

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s \
    CMD curl -sf http://localhost:3069/health || exit 1

CMD ["axum-api"]

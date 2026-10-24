# syntax=docker/dockerfile:1

# ---- Build stage ----
FROM rust:1.79-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies separately from source
COPY services/api/Cargo.toml services/api/Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin api 2>/dev/null || true
RUN rm -rf src

# Build actual source
COPY services/api/src ./src
COPY services/api/migrations ./migrations
RUN cargo build --release --bin api

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash appuser
USER appuser
WORKDIR /home/appuser

COPY --from=builder /app/target/release/api ./api
COPY --from=builder /app/migrations ./migrations

EXPOSE 3000

ENTRYPOINT ["./api"]

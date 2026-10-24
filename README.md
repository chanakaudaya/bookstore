# Bookstore — Multi-tenant Ecommerce Platform

A full-stack multi-tenant ecommerce application for selling books online.

## Stack

| Layer | Technology |
|---|---|
| Frontend | React + TypeScript + Vite |
| Backend | Rust (Axum) |
| Database | PostgreSQL |
| Cache | Redis |
| Identity | Keycloak |

## Monorepo layout

```
bookstore/
├── apps/web/          # React customer + admin UI
├── services/api/      # Rust HTTP API (modular monolith)
├── infra/             # docker-compose, Keycloak realm, scripts
└── docs/              # Architecture decision records (ADR)
```

## Quick start

### Prerequisites
- Docker and Docker Compose
- Rust toolchain (stable, see `rust-toolchain.toml`)
- Node.js 20+

### 1. Start infrastructure

```bash
cd infra
docker compose up -d
```

This starts PostgreSQL (port 5432), Redis (port 6379), and Keycloak (port 8080).

Keycloak admin console: http://localhost:8080 (admin / admin)

### 2. Run database migrations

```bash
cd services/api
cargo run --bin migrate
```

### 3. Start the API

```bash
cd services/api
cargo run
```

API runs on http://localhost:3000. Health check: `GET /health`

### 4. Start the web app

```bash
cd apps/web
npm install
npm run dev
```

Web app runs on http://localhost:5173.

## Environment variables

Copy `services/api/.env.example` to `services/api/.env` and `apps/web/.env.example` to `apps/web/.env` before running.

## Roles

| Role | Description |
|---|---|
| `customer` | Browse catalog, manage cart, checkout, manage account |
| `tenant_admin` | Manage catalog and orders within their tenant |
| `platform_admin` | Cross-tenant operations (future) |

## Development

See [docs/adr/](docs/adr/) for architectural decisions.

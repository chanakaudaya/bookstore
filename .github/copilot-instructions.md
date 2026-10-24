# Copilot Instructions — Bookstore

## Architecture overview

Multi-tenant ecommerce bookstore. Two app surfaces share one Rust API.

```
bookstore/
├── apps/web/          # React + TypeScript (Vite). Customer and admin UX.
├── services/api/      # Rust (Axum). Modular monolith, versioned REST at /api/v1.
├── infra/             # docker-compose (PostgreSQL 16, Redis 7, Keycloak 24).
└── docs/adr/          # Architecture decision records.
```

## Commands

### API (Rust) — run from `services/api/`
```bash
cargo run                        # start API on :3000
cargo run --bin migrate          # apply DB migrations
cargo test                       # full test suite
cargo test cart_tests            # single test module (e.g. cart_tests, checkout_tests, tenancy_tests, error_tests)
cargo test cart_tests::subtotal  # single test by name
cargo clippy -- -D warnings      # lint
cargo fmt                        # format
```

### Web (React) — run from `apps/web/`
```bash
npm run dev          # start dev server on :5173
npm run build        # type-check + build
npm run type-check   # tsc --noEmit only
npm run lint         # ESLint
npm test             # vitest (watch)
npm test -- --run    # vitest (single run, for CI)
npm test -- --run src/components/common/ProtectedRoute.test.tsx  # single test file
npm run test:coverage
```

### Infrastructure — run from `infra/`
```bash
docker compose up -d    # start postgres + redis + keycloak
docker compose down     # stop (keeps volumes)
docker compose down -v  # stop and delete volumes

# Production (from repo root)
docker compose -f infra/docker-compose.yml -f infra/docker-compose.prod.yml up -d
```

## Backend conventions (Rust)

**Module structure** — each domain (`catalog`, `users`, `cart`, `checkout`, `orders`) lives in `src/<domain>/mod.rs` and exports a `routes(state: AppState) -> Router<AppState>` function nested into the main router in `src/lib.rs`. Submodules (e.g. `users/provision.rs`) are declared via `pub mod` in the parent `mod.rs`.

**State** — `AppState` (in `src/common/mod.rs`) carries `db: PgPool`, `redis: ConnectionManager`, `jwks: JwksCache`, and `config`. All handlers receive it via `State(state): State<AppState>`.

**Error handling** — return `Result<_, AppError>` from every handler. `AppError` implements `IntoResponse` and maps to typed HTTP errors with a `{ "error": "CODE", "message": "..." }` JSON body.

**Response envelopes** — use `ApiResponse<T>` for single-item responses, `PagedResponse<T>` for lists. Both are in `src/common/mod.rs`.

**Multi-tenancy** — every handler that touches tenant-owned data must extract `TenantContext` (from `src/tenancy/mod.rs`). Parse `ctx.tenant_id` to a `Uuid` and include it in every database query. Never skip the tenant filter.

**Authentication** — routes use `.route_layer(middleware::from_fn_with_state(state, require_auth))`. The middleware validates the Keycloak JWT and inserts `AuthenticatedUser` as a request extension. The `auto_provision_profile` middleware (in `users/provision.rs`) runs after auth on the entire `/api/v1` prefix and upserts the user profile from JWT claims.

**Admin guards** — call `require_admin(&user)?` at the top of admin handlers. It checks for `tenant_admin` or `platform_admin` role.

**Audit logging** — use `log_audit(db, tenant_id, actor_id, action, entity_type, entity_id)` in `catalog/mod.rs` after any admin write. Writes to the `audit_events` table.

**Database** — use `sqlx::query_as!` and `sqlx::query!` macros (compile-time checked). Migrations live in `services/api/migrations/` and run via `cargo run --bin migrate`.

**Prices** — stored as `BIGINT` in the smallest currency unit (cents). Never store floats.

**Idempotency** — checkout uses a client-supplied `idempotency_key` (UUID). Check for an existing order with that key before creating a new one.

**Tests** — unit tests live in `src/tests/mod.rs` and test pure logic (cart subtotals, mock payment, role checks, error HTTP status codes). Use `axum-test` for integration tests that need an HTTP layer.

## Frontend conventions (React)

**API calls** — all API functions are in `src/services/api.ts` grouped by domain (`catalogApi`, `cartApi`, `checkoutApi`, `usersApi`, `ordersApi`). Use `@tanstack/react-query` for server state: `useQuery` for reads, `useMutation` for writes. `catalogApi` includes admin write methods (`createBook`, `updateBook`, `deleteBook`, `createCategory`, `updateCategory`, `deleteCategory`).

**Auth** — consume `useAuth()` from `src/contexts/AuthContext.tsx`. It exposes `user`, `isAuthenticated`, `token`, `login()`, `logout()`, and `hasRole(role)`.

**Layout** — all authenticated routes are wrapped in `<Layout />` which renders the nav bar with role-aware links. Admin nav links only render when `hasRole('tenant_admin')` returns true.

**Route protection** — wrap routes with `<ProtectedRoute />`. Pass `requiredRole="tenant_admin"` for admin-only routes.

**Types** — all shared API types live in `src/types/index.ts`. Keep API response shapes aligned with Rust struct field names (snake_case from API, camelCase in TypeScript types via axios).

**Query keys** — follow the pattern `['resource']` for collections and `['resource', id]` for single items. Invalidate related keys after mutations.

**Tests** — use Vitest + Testing Library. Mock `@/contexts/AuthContext` and `@/services/api` via `vi.mock`. Test files co-located with components (`.test.tsx`). See `ProtectedRoute.test.tsx` and `CartPage.test.tsx` for examples.

## Database schema key facts

- All tenant-owned tables have a `tenant_id UUID NOT NULL` column with a FK to `tenants`.
- Prices stored as `BIGINT` (cents).
- `cart_items` has a `UNIQUE (cart_id, book_id)` constraint — use `ON CONFLICT DO UPDATE` to upsert quantities.
- `orders` has a `UNIQUE (idempotency_key, tenant_id)` constraint.
- `payment_methods` stores only safe metadata (`last_four`, `token_reference`) — never raw card numbers.
- `order_items` snapshots book title and price at time of purchase (denormalized by design).

## Production deployment

Build and run with Docker:

```bash
# From repo root
docker compose -f infra/docker-compose.yml -f infra/docker-compose.prod.yml up -d
```

The production compose adds: API container (multi-stage Rust build), Web container (Vite build + nginx with SPA fallback and API proxy), Keycloak in production mode (TLS), and password env vars enforced via `${VAR:?err}`.

Create a `.env` file at repo root with: `POSTGRES_PASSWORD`, `REDIS_PASSWORD`, `KEYCLOAK_ADMIN_PASSWORD`, `KEYCLOAK_HOSTNAME`.

## Keycloak integration

- Realm: `bookstore`; Client: `bookstore-web` (public, PKCE S256).
- JWT claims used: `sub` (user ID), `tenant_id` (custom attribute), `realm_access.roles`.
- JWKS URL: `http://localhost:8080/realms/bookstore/protocol/openid-connect/certs`
- Demo users: `customer@example.com` / `demo123` and `admin@example.com` / `admin123` (tenant `tenant-demo-001`).

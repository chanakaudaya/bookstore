# Operations Runbook

## Starting the application locally

```bash
# 1. Start infrastructure
cd infra
docker compose up -d

# 2. Apply migrations
cd ../services/api
cp .env.example .env     # fill in values if needed
cargo run --bin migrate

# 3. Start API
cargo run

# 4. Start web (new terminal)
cd ../../apps/web
cp .env.example .env
npm install
npm run dev
```

Health check: `curl http://localhost:3000/health`
Metrics: `curl http://localhost:3000/metrics`

---

## Production deployment

### Prerequisites
- Docker and Docker Compose on the target host
- DNS record for `KEYCLOAK_HOSTNAME` pointing to the host
- TLS certificate (e.g. from Let's Encrypt) at `/etc/keycloak/tls.{crt,key}`

### First deploy

```bash
cp .env.prod.example .env
# Edit .env and fill in all secret values

docker compose -f infra/docker-compose.yml -f infra/docker-compose.prod.yml build
docker compose -f infra/docker-compose.yml -f infra/docker-compose.prod.yml up -d

# Run migrations once services are up
docker compose exec api ./migrate
```

### Upgrading

```bash
git pull
docker compose -f infra/docker-compose.yml -f infra/docker-compose.prod.yml build api web
docker compose -f infra/docker-compose.yml -f infra/docker-compose.prod.yml up -d --no-deps api web

# If there are new migrations
docker compose exec api ./migrate
```

### Rolling back a migration

Migrations are **forward-only** in this project. To roll back:
1. Deploy the previous image version.
2. Apply the inverse SQL manually via `psql` if needed.
3. Document the manual step as a new migration file.

---

## Adding a new tenant

1. Create a Keycloak user in the `bookstore` realm.
2. Set the `tenant_id` user attribute to the tenant UUID (must match an existing row in the `tenants` table).
3. Assign the `customer` or `tenant_admin` role.
4. Insert a `tenants` row if not already present:

```sql
INSERT INTO tenants (id, name, slug)
VALUES ('your-tenant-uuid', 'My Bookstore', 'my-bookstore');
```

The user profile will be auto-provisioned on their first authenticated API request.

---

## Monitoring

- **Health endpoint**: `GET /health` — returns `{"status":"ok"|"degraded", "checks":{"database":..., "cache":...}}`
- **Metrics endpoint**: `GET /metrics` — Prometheus text format with DB pool stats and build info
- **Structured logs**: JSON format. Filter with `jq`:
  ```bash
  docker compose logs api -f | jq 'select(.level == "ERROR")'
  ```

---

## Useful database queries

```sql
-- All tenants
SELECT id, name, slug, is_active FROM tenants;

-- Order counts by tenant and status
SELECT tenant_id, status, COUNT(*) FROM orders GROUP BY tenant_id, status ORDER BY tenant_id;

-- Recent audit events
SELECT actor_id, action, entity_type, entity_id, created_at
FROM audit_events
ORDER BY created_at DESC
LIMIT 50;

-- Users without a profile (should be empty if auto-provision works)
SELECT keycloak_user_id, email FROM user_profiles WHERE email = '';
```

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| API returns 401 on all requests | JWKS URL unreachable or wrong issuer | Check `KEYCLOAK_JWKS_URL` and `KEYCLOAK_ISSUER` env vars |
| API returns 400 "Missing tenant_id claim" | User missing `tenant_id` attribute in Keycloak | Set `tenant_id` user attribute in Keycloak admin console |
| Cart returns empty after login | `user_profiles` row missing for this user | Should auto-provision; check API logs for provision errors |
| Checkout returns "Cart is empty" | Cart was cleared after a successful order (expected) | Place items in cart again |
| Migration fails with "relation already exists" | Migrations table out of sync | Check `_sqlx_migrations` table; manually mark applied |

# ADR-001: Multi-tenancy Strategy

**Status:** Accepted

## Context

The bookstore platform must serve multiple independent tenants (stores) from a single deployment.

## Decision

Use a **shared-schema, row-level tenant isolation** model:

- Every tenant-owned entity table has a `tenant_id UUID NOT NULL` column with a FK to the `tenants` table.
- The `tenant_id` is propagated into the JWT as a custom claim from Keycloak.
- The API resolves tenant context from the JWT via the `TenantContext` extractor.
- Every database query against a tenant-owned table **must** include `WHERE tenant_id = $n`.
- The middleware + repository layers enforce this; no handler can opt out.

## Consequences

- Simpler ops: one DB instance, one schema, one deployment.
- Tenant data isolation enforced at the application layer — requires discipline in every query.
- Migrations are simpler than schema-per-tenant.
- Suitable for MVP scale; can migrate to schema-per-tenant or DB-per-tenant later if isolation requirements increase.

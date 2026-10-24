use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use uuid::Uuid;

use crate::authz::AuthenticatedUser;
use crate::common::{AppError, AppState};

/// Represents the current tenant context resolved from the JWT.
/// `tenant_id` is always a valid UUID string by the time it reaches a handler.
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: String,
    pub tenant_uuid: Uuid,
    pub user_id: String,
}

impl TenantContext {
    pub fn new(tenant_id: impl Into<String>, tenant_uuid: Uuid, user_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            tenant_uuid,
            user_id: user_id.into(),
        }
    }
}

/// Axum extractor that resolves the `TenantContext` from request extensions.
/// Accepts both UUID strings and slugs — slugs are resolved to a UUID via DB lookup.
/// Requires `require_auth` middleware and `AppState` to have been set up.
#[axum::async_trait]
impl FromRequestParts<AppState> for TenantContext {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<AuthenticatedUser>()
            .ok_or(AppError::Unauthorized)?;

        let raw = user
            .0
            .tenant_id
            .clone()
            .ok_or_else(|| AppError::BadRequest("Missing tenant_id claim".to_string()))?;

        // Try to parse as UUID directly; if that fails, look up by slug.
        let uuid = if let Ok(u) = Uuid::parse_str(&raw) {
            // Verify the tenant exists
            let exists: bool = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM tenants WHERE id = $1 AND is_active = true)",
                u
            )
            .fetch_one(&state.db)
            .await?
            .unwrap_or(false);

            if !exists {
                return Err(AppError::BadRequest(format!("Tenant {raw} not found")));
            }
            u
        } else {
            // Treat as slug
            let id: Option<Uuid> = sqlx::query_scalar!(
                "SELECT id FROM tenants WHERE slug = $1 AND is_active = true",
                raw
            )
            .fetch_optional(&state.db)
            .await?;

            id.ok_or_else(|| AppError::BadRequest(format!("Tenant '{raw}' not found")))?
        };

        Ok(TenantContext::new(uuid.to_string(), uuid, user.0.sub.clone()))
    }
}

/// Guard: requires `tenant_admin` or `platform_admin` role. Returns Forbidden otherwise.
pub fn require_admin(user: &AuthenticatedUser) -> Result<(), AppError> {
    if user.0.is_tenant_admin() || user.0.is_platform_admin() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

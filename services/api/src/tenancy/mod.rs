use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::authz::AuthenticatedUser;
use crate::common::AppError;

/// Represents the current tenant context resolved from the JWT.
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: String,
    pub user_id: String,
}

impl TenantContext {
    pub fn new(tenant_id: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self {
            tenant_id: tenant_id.into(),
            user_id: user_id.into(),
        }
    }
}

/// Axum extractor that resolves the `TenantContext` from request extensions.
/// Requires `require_auth` middleware to have run first.
#[axum::async_trait]
impl<S: Send + Sync> FromRequestParts<S> for TenantContext {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<AuthenticatedUser>()
            .ok_or(AppError::Unauthorized)?;

        let tenant_id = user
            .0
            .tenant_id
            .clone()
            .ok_or_else(|| AppError::BadRequest("Missing tenant_id claim".to_string()))?;

        Ok(TenantContext::new(tenant_id, user.0.sub.clone()))
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

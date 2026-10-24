use axum::{extract::Request, middleware::Next, response::Response};
use tracing::warn;
use uuid::Uuid;

use crate::authz::AuthenticatedUser;
use crate::common::{AppError, AppState};

/// Middleware that auto-provisions a `user_profiles` row from JWT claims on first
/// encounter. Resolves tenant by UUID or slug. Must run after `require_auth`.
pub async fn auto_provision_profile(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
        let user = user.0.clone();

        if let Some(tenant_id_str) = &user.tenant_id {
            // Resolve UUID or slug → UUID
            let tenant_uuid: Option<Uuid> = if let Ok(u) = Uuid::parse_str(tenant_id_str) {
                Some(u)
            } else {
                sqlx::query_scalar!(
                    "SELECT id FROM tenants WHERE slug = $1 AND is_active = true",
                    tenant_id_str
                )
                .fetch_optional(&state.db)
                .await
                .unwrap_or(None)
            };

            if let Some(tenant_uuid) = tenant_uuid {
                let email = user.email.clone().unwrap_or_default();
                let upsert_result = sqlx::query!(
                    r#"
                    INSERT INTO user_profiles (id, tenant_id, keycloak_user_id, email)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (tenant_id, keycloak_user_id) DO UPDATE
                        SET email = EXCLUDED.email
                    "#,
                    Uuid::new_v4(),
                    tenant_uuid,
                    user.sub,
                    email,
                )
                .execute(&state.db)
                .await;

                if let Err(e) = upsert_result {
                    warn!(error = %e, sub = %user.sub, "Profile auto-provision failed");
                }
            } else {
                warn!(tenant = %tenant_id_str, sub = %user.sub, "Tenant not found during auto-provision");
            }
        }
    }

    Ok(next.run(req).await)
}

use axum::{extract::Request, middleware::Next, response::Response};
use tracing::warn;
use uuid::Uuid;

use crate::authz::AuthenticatedUser;
use crate::common::{AppError, AppState};

/// Middleware that auto-provisions a `user_profiles` row from JWT claims on first
/// encounter. Must run after `require_auth` so `AuthenticatedUser` is in extensions.
pub async fn auto_provision_profile(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
        let user = user.0.clone();

        if let Some(tenant_id_str) = &user.tenant_id {
            if let Ok(tenant_uuid) = Uuid::parse_str(tenant_id_str) {
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
            }
        }
    }

    Ok(next.run(req).await)
}

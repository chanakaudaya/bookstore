pub mod provision;

use axum::{
    extract::{Path, State},
    middleware,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::authz::{require_auth, AuthenticatedUser};
use crate::common::{AppError, AppState, ApiResponse};
use crate::tenancy::TenantContext;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub keycloak_user_id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ShippingAddress {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub label: Option<String>,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAddressRequest {
    pub label: Option<String>,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
    pub is_default: Option<bool>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PaymentMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub label: String,
    pub method_type: String,
    pub last_four: Option<String>,
    pub token_reference: Option<String>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePaymentMethodRequest {
    pub label: String,
    pub method_type: String,
    pub last_four: Option<String>,
    pub token_reference: Option<String>,
    pub is_default: Option<bool>,
}

// ---------------------------------------------------------------------------
// Handlers — Profile
// ---------------------------------------------------------------------------

async fn get_my_profile(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<UserProfile>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile = sqlx::query_as!(
        UserProfile,
        r#"
        SELECT id, tenant_id, keycloak_user_id, email, first_name, last_name, phone, created_at, updated_at
        FROM user_profiles
        WHERE keycloak_user_id = $1 AND tenant_id = $2
        "#,
        user.0.sub,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(profile))
}

async fn update_my_profile(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile = sqlx::query_as!(
        UserProfile,
        r#"
        UPDATE user_profiles
        SET first_name = COALESCE($3, first_name),
            last_name  = COALESCE($4, last_name),
            phone      = COALESCE($5, phone),
            updated_at = NOW()
        WHERE keycloak_user_id = $1 AND tenant_id = $2
        RETURNING id, tenant_id, keycloak_user_id, email, first_name, last_name, phone, created_at, updated_at
        "#,
        user.0.sub,
        tenant_uuid,
        payload.first_name,
        payload.last_name,
        payload.phone
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(profile))
}

// ---------------------------------------------------------------------------
// Handlers — Addresses
// ---------------------------------------------------------------------------

async fn list_addresses(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<ShippingAddress>>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM user_profiles WHERE keycloak_user_id = $1 AND tenant_id = $2",
        user.0.sub,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let addresses = sqlx::query_as!(
        ShippingAddress,
        "SELECT id, user_id, tenant_id, label, line1, line2, city, state, postal_code, country, is_default, created_at FROM shipping_addresses WHERE user_id = $1 AND tenant_id = $2 ORDER BY is_default DESC, created_at",
        profile_id,
        tenant_uuid
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(addresses))
}

async fn create_address(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateAddressRequest>,
) -> Result<Json<ShippingAddress>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM user_profiles WHERE keycloak_user_id = $1 AND tenant_id = $2",
        user.0.sub,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let address = sqlx::query_as!(
        ShippingAddress,
        r#"
        INSERT INTO shipping_addresses (id, user_id, tenant_id, label, line1, line2, city, state, postal_code, country, is_default)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, user_id, tenant_id, label, line1, line2, city, state, postal_code, country, is_default, created_at
        "#,
        Uuid::new_v4(),
        profile_id,
        tenant_uuid,
        payload.label,
        payload.line1,
        payload.line2,
        payload.city,
        payload.state,
        payload.postal_code,
        payload.country,
        payload.is_default.unwrap_or(false)
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(address))
}

async fn delete_address(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Path(address_id): Path<Uuid>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM user_profiles WHERE keycloak_user_id = $1 AND tenant_id = $2",
        user.0.sub,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let rows = sqlx::query!(
        "DELETE FROM shipping_addresses WHERE id = $1 AND user_id = $2 AND tenant_id = $3",
        address_id,
        profile_id,
        tenant_uuid
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(ApiResponse::new("deleted")))
}

// ---------------------------------------------------------------------------
// Handlers — Payment Methods
// ---------------------------------------------------------------------------

async fn list_payment_methods(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<PaymentMethod>>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM user_profiles WHERE keycloak_user_id = $1 AND tenant_id = $2",
        user.0.sub,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let methods = sqlx::query_as!(
        PaymentMethod,
        "SELECT id, user_id, tenant_id, label, method_type, last_four, token_reference, is_default, created_at FROM payment_methods WHERE user_id = $1 AND tenant_id = $2 ORDER BY is_default DESC, created_at",
        profile_id,
        tenant_uuid
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(methods))
}

async fn create_payment_method(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreatePaymentMethodRequest>,
) -> Result<Json<PaymentMethod>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM user_profiles WHERE keycloak_user_id = $1 AND tenant_id = $2",
        user.0.sub,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let method = sqlx::query_as!(
        PaymentMethod,
        r#"
        INSERT INTO payment_methods (id, user_id, tenant_id, label, method_type, last_four, token_reference, is_default)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, user_id, tenant_id, label, method_type, last_four, token_reference, is_default, created_at
        "#,
        Uuid::new_v4(),
        profile_id,
        tenant_uuid,
        payload.label,
        payload.method_type,
        payload.last_four,
        payload.token_reference,
        payload.is_default.unwrap_or(false)
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(method))
}

async fn delete_payment_method(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Path(method_id): Path<Uuid>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM user_profiles WHERE keycloak_user_id = $1 AND tenant_id = $2",
        user.0.sub,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let rows = sqlx::query!(
        "DELETE FROM payment_methods WHERE id = $1 AND user_id = $2 AND tenant_id = $3",
        method_id,
        profile_id,
        tenant_uuid
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(ApiResponse::new("deleted")))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(get_my_profile).put(update_my_profile))
        .route("/me/addresses", get(list_addresses).post(create_address))
        .route("/me/addresses/:address_id", delete(delete_address))
        .route("/me/payment-methods", get(list_payment_methods).post(create_payment_method))
        .route("/me/payment-methods/:method_id", delete(delete_payment_method))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}

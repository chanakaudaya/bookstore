use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::{get, put},
    Extension, Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::authz::{require_auth, AuthenticatedUser};
use crate::common::{AppError, AppState, PagedResponse};
use crate::tenancy::{require_admin, TenantContext};

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub payment_status: String,
    pub total: i64,
    pub shipping_address_id: Uuid,
    pub payment_method_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub book_id: Uuid,
    pub title: String,
    pub unit_price: i64,
    pub quantity: i32,
}

#[derive(Debug, Serialize)]
pub struct OrderWithItems {
    #[serde(flatten)]
    pub order: Order,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Deserialize)]
pub struct OrderListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: String,
}

// ---------------------------------------------------------------------------
// Handlers — Customer
// ---------------------------------------------------------------------------

async fn list_my_orders(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<OrderListQuery>,
) -> Result<Json<PagedResponse<Order>>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM user_profiles WHERE keycloak_user_id = $1 AND tenant_id = $2",
        user.0.sub,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let orders = sqlx::query_as!(
        Order,
        "SELECT id, tenant_id, user_id, status, payment_status, total, shipping_address_id, payment_method_id, created_at, updated_at FROM orders WHERE user_id = $1 AND tenant_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
        profile_id,
        tenant_uuid,
        per_page,
        offset
    )
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM orders WHERE user_id = $1 AND tenant_id = $2",
        profile_id,
        tenant_uuid
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);

    Ok(Json(PagedResponse { data: orders, total, page, per_page }))
}

async fn get_my_order(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<OrderWithItems>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM user_profiles WHERE keycloak_user_id = $1 AND tenant_id = $2",
        user.0.sub,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let order = sqlx::query_as!(
        Order,
        "SELECT id, tenant_id, user_id, status, payment_status, total, shipping_address_id, payment_method_id, created_at, updated_at FROM orders WHERE id = $1 AND user_id = $2 AND tenant_id = $3",
        order_id,
        profile_id,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let items = sqlx::query_as!(
        OrderItem,
        "SELECT id, order_id, book_id, title, unit_price, quantity FROM order_items WHERE order_id = $1",
        order_id
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(OrderWithItems { order, items }))
}

// ---------------------------------------------------------------------------
// Handlers — Admin
// ---------------------------------------------------------------------------

async fn admin_list_orders(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<OrderListQuery>,
) -> Result<Json<PagedResponse<Order>>, AppError> {
    require_admin(&user)?;

    let tenant_uuid = ctx.tenant_uuid;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let orders = sqlx::query_as!(
        Order,
        "SELECT id, tenant_id, user_id, status, payment_status, total, shipping_address_id, payment_method_id, created_at, updated_at FROM orders WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        tenant_uuid,
        per_page,
        offset
    )
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM orders WHERE tenant_id = $1",
        tenant_uuid
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);

    Ok(Json(PagedResponse { data: orders, total, page, per_page }))
}

async fn admin_update_order_status(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Path(order_id): Path<Uuid>,
    Json(payload): Json<UpdateOrderStatusRequest>,
) -> Result<Json<Order>, AppError> {
    require_admin(&user)?;

    let valid_statuses = ["confirmed", "processing", "shipped", "delivered", "cancelled"];
    if !valid_statuses.contains(&payload.status.as_str()) {
        return Err(AppError::BadRequest(format!("Invalid status: {}", payload.status)));
    }

    let tenant_uuid = ctx.tenant_uuid;

    let order = sqlx::query_as!(
        Order,
        "UPDATE orders SET status = $1, updated_at = NOW() WHERE id = $2 AND tenant_id = $3 RETURNING id, tenant_id, user_id, status, payment_status, total, shipping_address_id, payment_method_id, created_at, updated_at",
        payload.status,
        order_id,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(order))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(list_my_orders))
        .route("/me/:order_id", get(get_my_order))
        .route("/admin", get(admin_list_orders))
        .route("/admin/:order_id/status", put(admin_update_order_status))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}

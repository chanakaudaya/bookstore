use axum::{
    extract::State,
    middleware,
    routing::{delete, get, post},
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Cart {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CartItem {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub book_id: Uuid,
    pub title: String,
    pub unit_price: i64,
    pub quantity: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CartWithItems {
    #[serde(flatten)]
    pub cart: Cart,
    pub items: Vec<CartItem>,
    pub subtotal: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpsertCartItemRequest {
    pub book_id: Uuid,
    pub quantity: i32,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn resolve_profile_id(
    db: &sqlx::PgPool,
    keycloak_user_id: &str,
    tenant_id: Uuid,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar!(
        "SELECT id FROM user_profiles WHERE keycloak_user_id = $1 AND tenant_id = $2",
        keycloak_user_id,
        tenant_id
    )
    .fetch_optional(db)
    .await?
    .ok_or(AppError::NotFound)
}

async fn get_or_create_cart(
    db: &sqlx::PgPool,
    user_id: Uuid,
    tenant_id: Uuid,
) -> Result<Cart, AppError> {
    if let Some(cart) = sqlx::query_as!(
        Cart,
        "SELECT id, user_id, tenant_id, created_at, updated_at FROM carts WHERE user_id = $1 AND tenant_id = $2",
        user_id,
        tenant_id
    )
    .fetch_optional(db)
    .await? {
        return Ok(cart);
    }

    Ok(sqlx::query_as!(
        Cart,
        "INSERT INTO carts (id, user_id, tenant_id) VALUES ($1, $2, $3) RETURNING id, user_id, tenant_id, created_at, updated_at",
        Uuid::new_v4(),
        user_id,
        tenant_id
    )
    .fetch_one(db)
    .await?)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn get_cart(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<CartWithItems>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile_id = resolve_profile_id(&state.db, &user.0.sub, tenant_uuid).await?;
    let cart = get_or_create_cart(&state.db, profile_id, tenant_uuid).await?;

    let items = sqlx::query_as!(
        CartItem,
        "SELECT id, cart_id, book_id, title, unit_price, quantity, created_at FROM cart_items WHERE cart_id = $1 ORDER BY created_at",
        cart.id
    )
    .fetch_all(&state.db)
    .await?;

    let subtotal: i64 = items.iter().map(|i| i.unit_price * i.quantity as i64).sum();

    Ok(Json(CartWithItems { cart, items, subtotal }))
}

async fn upsert_cart_item(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<UpsertCartItemRequest>,
) -> Result<Json<CartWithItems>, AppError> {
    if payload.quantity < 0 {
        return Err(AppError::BadRequest("Quantity must be >= 0".into()));
    }

    let tenant_uuid = ctx.tenant_uuid;

    let profile_id = resolve_profile_id(&state.db, &user.0.sub, tenant_uuid).await?;
    let cart = get_or_create_cart(&state.db, profile_id, tenant_uuid).await?;

    let book = sqlx::query!(
        "SELECT id, title, price, stock_qty FROM books WHERE id = $1 AND tenant_id = $2",
        payload.book_id,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    if payload.quantity == 0 {
        sqlx::query!(
            "DELETE FROM cart_items WHERE cart_id = $1 AND book_id = $2",
            cart.id,
            payload.book_id
        )
        .execute(&state.db)
        .await?;
    } else {
        if payload.quantity > book.stock_qty {
            return Err(AppError::BadRequest(format!(
                "Only {} items in stock",
                book.stock_qty
            )));
        }

        sqlx::query!(
            r#"
            INSERT INTO cart_items (id, cart_id, book_id, title, unit_price, quantity)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (cart_id, book_id) DO UPDATE
              SET quantity = $6, unit_price = $5
            "#,
            Uuid::new_v4(),
            cart.id,
            payload.book_id,
            book.title,
            book.price,
            payload.quantity
        )
        .execute(&state.db)
        .await?;
    }

    // Return fresh cart
    let items = sqlx::query_as!(
        CartItem,
        "SELECT id, cart_id, book_id, title, unit_price, quantity, created_at FROM cart_items WHERE cart_id = $1 ORDER BY created_at",
        cart.id
    )
    .fetch_all(&state.db)
    .await?;

    let subtotal: i64 = items.iter().map(|i| i.unit_price * i.quantity as i64).sum();

    Ok(Json(CartWithItems { cart, items, subtotal }))
}

async fn clear_cart(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    let tenant_uuid = ctx.tenant_uuid;

    let profile_id = resolve_profile_id(&state.db, &user.0.sub, tenant_uuid).await?;

    sqlx::query!(
        "DELETE FROM cart_items WHERE cart_id IN (SELECT id FROM carts WHERE user_id = $1 AND tenant_id = $2)",
        profile_id,
        tenant_uuid
    )
    .execute(&state.db)
    .await?;

    Ok(Json(ApiResponse::new("cleared")))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_cart).post(upsert_cart_item).delete(clear_cart))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}

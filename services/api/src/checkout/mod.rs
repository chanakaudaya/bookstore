use axum::{
    extract::State,
    middleware,
    routing::post,
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::authz::{require_auth, AuthenticatedUser};
use crate::common::{AppError, AppState};
use crate::tenancy::TenantContext;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CheckoutRequest {
    /// Client-generated idempotency key to prevent duplicate orders.
    pub idempotency_key: Uuid,
    pub shipping_address_id: Uuid,
    pub payment_method_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub order_id: Uuid,
    pub status: String,
    pub payment_status: String,
    pub total: i64,
}

// ---------------------------------------------------------------------------
// Mock payment processor
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum MockPaymentResult {
    Approved,
    Declined,
}

fn mock_process_payment(amount: i64) -> MockPaymentResult {
    // In real implementation, call payment gateway.
    // For MVP: always approve when amount > 0.
    if amount > 0 {
        MockPaymentResult::Approved
    } else {
        MockPaymentResult::Declined
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

async fn checkout(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CheckoutRequest>,
) -> Result<Json<CheckoutResponse>, AppError> {
    let tenant_uuid = Uuid::parse_str(&ctx.tenant_id)
        .map_err(|_| AppError::BadRequest("Invalid tenant_id".into()))?;

    // Idempotency: return existing order if this key was already used
    if let Some(existing_order_id) = sqlx::query_scalar!(
        "SELECT id FROM orders WHERE idempotency_key = $1 AND tenant_id = $2",
        payload.idempotency_key,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .flatten()
    {
        let order = sqlx::query!(
            "SELECT id, status, payment_status, total FROM orders WHERE id = $1",
            existing_order_id
        )
        .fetch_one(&state.db)
        .await?;

        return Ok(Json(CheckoutResponse {
            order_id: order.id,
            status: order.status,
            payment_status: order.payment_status,
            total: order.total,
        }));
    }

    // Resolve user profile
    let profile_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM user_profiles WHERE keycloak_user_id = $1 AND tenant_id = $2",
        user.0.sub,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .flatten()
    .ok_or(AppError::NotFound)?;

    // Validate shipping address belongs to user
    let _addr = sqlx::query!(
        "SELECT id FROM shipping_addresses WHERE id = $1 AND user_id = $2 AND tenant_id = $3",
        payload.shipping_address_id,
        profile_id,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("Shipping address not found".into()))?;

    // Validate payment method belongs to user
    let _pm = sqlx::query!(
        "SELECT id FROM payment_methods WHERE id = $1 AND user_id = $2 AND tenant_id = $3",
        payload.payment_method_id,
        profile_id,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("Payment method not found".into()))?;

    // Fetch cart and items
    let cart = sqlx::query!(
        "SELECT id FROM carts WHERE user_id = $1 AND tenant_id = $2",
        profile_id,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("Cart is empty".into()))?;

    let cart_items = sqlx::query!(
        "SELECT book_id, title, unit_price, quantity FROM cart_items WHERE cart_id = $1",
        cart.id
    )
    .fetch_all(&state.db)
    .await?;

    if cart_items.is_empty() {
        return Err(AppError::BadRequest("Cart is empty".into()));
    }

    let total: i64 = cart_items.iter().map(|i| i.unit_price * i.quantity as i64).sum();

    // Mock payment
    let payment_status = match mock_process_payment(total) {
        MockPaymentResult::Approved => "approved",
        MockPaymentResult::Declined => "declined",
    };

    // Create order in a transaction
    let order_id = Uuid::new_v4();

    let mut tx = state.db.begin().await?;

    sqlx::query!(
        r#"
        INSERT INTO orders (id, tenant_id, user_id, idempotency_key, shipping_address_id,
                            payment_method_id, status, payment_status, total)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        order_id,
        tenant_uuid,
        profile_id,
        payload.idempotency_key,
        payload.shipping_address_id,
        payload.payment_method_id,
        if payment_status == "approved" { "confirmed" } else { "payment_failed" },
        payment_status,
        total
    )
    .execute(&mut *tx)
    .await?;

    for item in &cart_items {
        sqlx::query!(
            r#"
            INSERT INTO order_items (id, order_id, book_id, title, unit_price, quantity)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            Uuid::new_v4(),
            order_id,
            item.book_id,
            item.title,
            item.unit_price,
            item.quantity
        )
        .execute(&mut *tx)
        .await?;

        // Decrement stock (best-effort in MVP; a real system would lock rows)
        sqlx::query!(
            "UPDATE books SET stock_qty = stock_qty - $1 WHERE id = $2 AND tenant_id = $3",
            item.quantity,
            item.book_id,
            tenant_uuid
        )
        .execute(&mut *tx)
        .await?;
    }

    // Clear cart after successful order
    if payment_status == "approved" {
        sqlx::query!("DELETE FROM cart_items WHERE cart_id = $1", cart.id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    let order_status = if payment_status == "approved" {
        "confirmed"
    } else {
        "payment_failed"
    };

    Ok(Json(CheckoutResponse {
        order_id,
        status: order_status.to_string(),
        payment_status: payment_status.to_string(),
        total,
    }))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(checkout))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}

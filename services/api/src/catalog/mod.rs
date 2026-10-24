use axum::{
    extract::{Path, Query, State},
    middleware,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::authz::require_auth;
use crate::common::{AppError, AppState, PagedResponse};
use crate::tenancy::TenantContext;

// ---------------------------------------------------------------------------
// Models
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Category {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Book {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub category_id: Option<Uuid>,
    pub title: String,
    pub author: String,
    pub isbn: Option<String>,
    pub description: Option<String>,
    pub price: i64,
    pub stock_qty: i32,
    pub cover_image_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Query params
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BookListQuery {
    pub q: Option<String>,
    pub category_id: Option<Uuid>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn list_books(
    State(state): State<AppState>,
    ctx: TenantContext,
    Query(params): Query<BookListQuery>,
) -> Result<Json<PagedResponse<Book>>, AppError> {
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let tenant_uuid = Uuid::parse_str(&ctx.tenant_id)
        .map_err(|_| AppError::BadRequest("Invalid tenant_id".into()))?;

    let books = sqlx::query_as!(
        Book,
        r#"
        SELECT id, tenant_id, category_id, title, author, isbn,
               description, price, stock_qty, cover_image_url, published_at, created_at, updated_at
        FROM books
        WHERE tenant_id = $1
          AND ($2::text IS NULL OR title ILIKE '%' || $2 || '%' OR author ILIKE '%' || $2 || '%')
          AND ($3::uuid IS NULL OR category_id = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
        tenant_uuid,
        params.q,
        params.category_id,
        per_page,
        offset
    )
    .fetch_all(&state.db)
    .await?;

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM books WHERE tenant_id = $1",
        tenant_uuid
    )
    .fetch_one(&state.db)
    .await?
    .unwrap_or(0);

    Ok(Json(PagedResponse { data: books, total, page, per_page }))
}

async fn get_book(
    State(state): State<AppState>,
    ctx: TenantContext,
    Path(book_id): Path<Uuid>,
) -> Result<Json<Book>, AppError> {
    let tenant_uuid = Uuid::parse_str(&ctx.tenant_id)
        .map_err(|_| AppError::BadRequest("Invalid tenant_id".into()))?;

    let book = sqlx::query_as!(
        Book,
        r#"
        SELECT id, tenant_id, category_id, title, author, isbn,
               description, price, stock_qty, cover_image_url, published_at, created_at, updated_at
        FROM books
        WHERE id = $1 AND tenant_id = $2
        "#,
        book_id,
        tenant_uuid
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(book))
}

async fn list_categories(
    State(state): State<AppState>,
    ctx: TenantContext,
) -> Result<Json<Vec<Category>>, AppError> {
    let tenant_uuid = Uuid::parse_str(&ctx.tenant_id)
        .map_err(|_| AppError::BadRequest("Invalid tenant_id".into()))?;

    let categories = sqlx::query_as!(
        Category,
        "SELECT id, tenant_id, name, slug, description, created_at FROM categories WHERE tenant_id = $1 ORDER BY name",
        tenant_uuid
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(categories))
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/books", get(list_books))
        .route("/books/:book_id", get(get_book))
        .route("/categories", get(list_categories))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}

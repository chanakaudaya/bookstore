use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::authz::{require_auth, AuthenticatedUser};
use crate::common::{ApiResponse, AppError, AppState, PagedResponse};
use crate::tenancy::{require_admin, TenantContext};

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
    let tenant_uuid = ctx.tenant_uuid;

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
    let tenant_uuid = ctx.tenant_uuid;

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
    let tenant_uuid = ctx.tenant_uuid;

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
// Admin request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateBookRequest {
    pub category_id: Option<Uuid>,
    pub title: String,
    pub author: String,
    pub isbn: Option<String>,
    pub description: Option<String>,
    /// Price in cents.
    pub price: i64,
    pub stock_qty: i32,
    pub cover_image_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookRequest {
    pub category_id: Option<Uuid>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub description: Option<String>,
    pub price: Option<i64>,
    pub stock_qty: Option<i32>,
    pub cover_image_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Admin handlers — books
// ---------------------------------------------------------------------------

async fn admin_create_book(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateBookRequest>,
) -> Result<(StatusCode, Json<Book>), AppError> {
    require_admin(&user)?;

    if payload.price < 0 {
        return Err(AppError::BadRequest("Price must be non-negative".into()));
    }
    if payload.stock_qty < 0 {
        return Err(AppError::BadRequest("Stock quantity must be non-negative".into()));
    }

    let tenant_uuid = ctx.tenant_uuid;

    let book = sqlx::query_as!(
        Book,
        r#"
        INSERT INTO books (id, tenant_id, category_id, title, author, isbn, description,
                           price, stock_qty, cover_image_url, published_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, tenant_id, category_id, title, author, isbn,
                  description, price, stock_qty, cover_image_url, published_at, created_at, updated_at
        "#,
        Uuid::new_v4(),
        tenant_uuid,
        payload.category_id,
        payload.title,
        payload.author,
        payload.isbn,
        payload.description,
        payload.price,
        payload.stock_qty,
        payload.cover_image_url,
        payload.published_at
    )
    .fetch_one(&state.db)
    .await?;

    log_audit(&state.db, tenant_uuid, &user.0.sub, "book.created", "book", book.id).await;

    Ok((StatusCode::CREATED, Json(book)))
}

async fn admin_update_book(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Path(book_id): Path<Uuid>,
    Json(payload): Json<UpdateBookRequest>,
) -> Result<Json<Book>, AppError> {
    require_admin(&user)?;

    let tenant_uuid = ctx.tenant_uuid;

    let book = sqlx::query_as!(
        Book,
        r#"
        UPDATE books
        SET category_id     = COALESCE($3, category_id),
            title           = COALESCE($4, title),
            author          = COALESCE($5, author),
            isbn            = COALESCE($6, isbn),
            description     = COALESCE($7, description),
            price           = COALESCE($8, price),
            stock_qty       = COALESCE($9, stock_qty),
            cover_image_url = COALESCE($10, cover_image_url),
            published_at    = COALESCE($11, published_at),
            updated_at      = NOW()
        WHERE id = $1 AND tenant_id = $2
        RETURNING id, tenant_id, category_id, title, author, isbn,
                  description, price, stock_qty, cover_image_url, published_at, created_at, updated_at
        "#,
        book_id,
        tenant_uuid,
        payload.category_id,
        payload.title,
        payload.author,
        payload.isbn,
        payload.description,
        payload.price,
        payload.stock_qty,
        payload.cover_image_url,
        payload.published_at
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    log_audit(&state.db, tenant_uuid, &user.0.sub, "book.updated", "book", book.id).await;

    Ok(Json(book))
}

async fn admin_delete_book(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Path(book_id): Path<Uuid>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    require_admin(&user)?;

    let tenant_uuid = ctx.tenant_uuid;

    let rows = sqlx::query!(
        "DELETE FROM books WHERE id = $1 AND tenant_id = $2",
        book_id,
        tenant_uuid
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    log_audit(&state.db, tenant_uuid, &user.0.sub, "book.deleted", "book", book_id).await;

    Ok(Json(ApiResponse::new("deleted")))
}

// ---------------------------------------------------------------------------
// Admin handlers — categories
// ---------------------------------------------------------------------------

async fn admin_create_category(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<Category>), AppError> {
    require_admin(&user)?;

    let tenant_uuid = ctx.tenant_uuid;

    let category = sqlx::query_as!(
        Category,
        r#"
        INSERT INTO categories (id, tenant_id, name, slug, description)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, tenant_id, name, slug, description, created_at
        "#,
        Uuid::new_v4(),
        tenant_uuid,
        payload.name,
        payload.slug,
        payload.description
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("unique") {
            AppError::Conflict("A category with that slug already exists".into())
        } else {
            AppError::from(e)
        }
    })?;

    log_audit(&state.db, tenant_uuid, &user.0.sub, "category.created", "category", category.id).await;

    Ok((StatusCode::CREATED, Json(category)))
}

async fn admin_update_category(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Path(category_id): Path<Uuid>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<Category>, AppError> {
    require_admin(&user)?;

    let tenant_uuid = ctx.tenant_uuid;

    let category = sqlx::query_as!(
        Category,
        r#"
        UPDATE categories
        SET name        = COALESCE($3, name),
            description = COALESCE($4, description)
        WHERE id = $1 AND tenant_id = $2
        RETURNING id, tenant_id, name, slug, description, created_at
        "#,
        category_id,
        tenant_uuid,
        payload.name,
        payload.description
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    log_audit(&state.db, tenant_uuid, &user.0.sub, "category.updated", "category", category.id).await;

    Ok(Json(category))
}

async fn admin_delete_category(
    State(state): State<AppState>,
    ctx: TenantContext,
    Extension(user): Extension<AuthenticatedUser>,
    Path(category_id): Path<Uuid>,
) -> Result<Json<ApiResponse<&'static str>>, AppError> {
    require_admin(&user)?;

    let tenant_uuid = ctx.tenant_uuid;

    let rows = sqlx::query!(
        "DELETE FROM categories WHERE id = $1 AND tenant_id = $2",
        category_id,
        tenant_uuid
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    log_audit(&state.db, tenant_uuid, &user.0.sub, "category.deleted", "category", category_id).await;

    Ok(Json(ApiResponse::new("deleted")))
}

// ---------------------------------------------------------------------------
// Audit helper
// ---------------------------------------------------------------------------

async fn log_audit(
    db: &sqlx::PgPool,
    tenant_id: Uuid,
    actor_id: &str,
    action: &str,
    entity_type: &str,
    entity_id: Uuid,
) {
    let _ = sqlx::query!(
        "INSERT INTO audit_events (id, tenant_id, actor_id, action, entity_type, entity_id) VALUES ($1, $2, $3, $4, $5, $6)",
        Uuid::new_v4(),
        tenant_id,
        actor_id,
        action,
        entity_type,
        entity_id
    )
    .execute(db)
    .await;
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Public read routes (still require auth for tenant context)
        .route("/books", get(list_books).post(admin_create_book))
        .route("/books/:book_id", get(get_book).put(admin_update_book).delete(admin_delete_book))
        .route("/categories", get(list_categories).post(admin_create_category))
        .route("/categories/:category_id", put(admin_update_category).delete(admin_delete_category))
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}

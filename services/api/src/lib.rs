pub mod authz;
pub mod cache;
pub mod catalog;
pub mod cart;
pub mod checkout;
pub mod common;
pub mod config;
pub mod db;
pub mod orders;
pub mod tenancy;
pub mod users;

use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::AppConfig;

pub async fn app(config: AppConfig) -> Result<Router> {
    let db_pool = db::create_pool(&config.database_url).await?;
    let redis_client = cache::create_client(&config.redis_url).await?;
    let jwks = authz::JwksCache::new(&config.keycloak_jwks_url);

    let state = common::AppState {
        db: db_pool,
        redis: redis_client,
        jwks,
        config,
    };

    let router = Router::new()
        .route("/health", get(health_handler))
        .nest("/api/v1", api_router(state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    Ok(router)
}

fn api_router(state: common::AppState) -> Router<common::AppState> {
    Router::new()
        .nest("/catalog", catalog::routes(state.clone()))
        .nest("/users", users::routes(state.clone()))
        .nest("/cart", cart::routes(state.clone()))
        .nest("/checkout", checkout::routes(state.clone()))
        .nest("/orders", orders::routes(state))
}

async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok" }))
}

pub mod authz;
pub mod cache;
pub mod catalog;
pub mod cart;
pub mod checkout;
pub mod common;
pub mod config;
pub mod db;
pub mod observability;
pub mod orders;
pub mod tenancy;
pub mod users;

#[cfg(test)]
mod tests;

use anyhow::Result;
use axum::{
    middleware,
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::AppConfig;
use crate::users::provision::auto_provision_profile;

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
        .merge(observability::routes(state.clone()))
        .nest(
            "/api/v1",
            api_router(state.clone())
                .layer(middleware::from_fn_with_state(state.clone(), auto_provision_profile)),
        )
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

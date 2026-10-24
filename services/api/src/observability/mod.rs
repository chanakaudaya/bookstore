use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use std::time::Instant;
use tracing::warn;

use crate::common::AppState;

// ---------------------------------------------------------------------------
// Health check
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub checks: HealthChecks,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct HealthChecks {
    pub database: ComponentHealth,
    pub cache: ComponentHealth,
}

#[derive(Serialize)]
pub struct ComponentHealth {
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub latency_ms: u128,
}

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let db_health = check_db(&state).await;
    let cache_health = check_cache(&state).await;

    let overall = if db_health.status == "ok" && cache_health.status == "ok" {
        "ok"
    } else {
        "degraded"
    };

    Json(HealthResponse {
        status: overall,
        version: env!("CARGO_PKG_VERSION"),
        checks: HealthChecks {
            database: db_health,
            cache: cache_health,
        },
    })
}

async fn check_db(state: &AppState) -> ComponentHealth {
    let start = Instant::now();
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => ComponentHealth {
            status: "ok",
            error: None,
            latency_ms: start.elapsed().as_millis(),
        },
        Err(e) => {
            warn!(error = %e, "Database health check failed");
            ComponentHealth {
                status: "error",
                error: Some(e.to_string()),
                latency_ms: start.elapsed().as_millis(),
            }
        }
    }
}

async fn check_cache(state: &AppState) -> ComponentHealth {
    let start = Instant::now();
    let mut conn = state.redis.clone();
    match redis::cmd("PING").query_async::<String>(&mut conn).await {
        Ok(_) => ComponentHealth {
            status: "ok",
            error: None,
            latency_ms: start.elapsed().as_millis(),
        },
        Err(e) => {
            warn!(error = %e, "Cache health check failed");
            ComponentHealth {
                status: "error",
                error: Some(e.to_string()),
                latency_ms: start.elapsed().as_millis(),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Prometheus-compatible metrics endpoint
// ---------------------------------------------------------------------------

async fn metrics_handler(State(state): State<AppState>) -> String {
    let db_pool_size = state.db.size();
    let db_idle = state.db.num_idle();

    format!(
        "# HELP bookstore_db_pool_size Total connections in the database pool\n\
         # TYPE bookstore_db_pool_size gauge\n\
         bookstore_db_pool_size {db_pool_size}\n\
         # HELP bookstore_db_pool_idle Idle connections in the database pool\n\
         # TYPE bookstore_db_pool_idle gauge\n\
         bookstore_db_pool_idle {db_idle}\n\
         # HELP bookstore_info Build information\n\
         # TYPE bookstore_info gauge\n\
         bookstore_info{{version=\"{}\"}} 1\n",
        env!("CARGO_PKG_VERSION")
    )
}

// ---------------------------------------------------------------------------
// Router — mounted at root level, not under /api/v1
// ---------------------------------------------------------------------------

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state)
}

use api::app;
use api::config::AppConfig;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "api=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = AppConfig::from_env()?;
    let bind_addr = format!("{}:{}", config.host, config.port);

    let router = app(config).await?;

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("API listening on {bind_addr}");

    axum::serve(listener, router).await?;
    Ok(())
}

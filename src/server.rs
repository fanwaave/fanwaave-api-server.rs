#![forbid(unsafe_code)]

use std::io;
use std::time::Duration;

use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, warn};

use crate::config::ApiConfig;
use crate::routes;
use crate::state::AppState;

pub async fn run(config: &ApiConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let database_url = config.database_url.as_deref().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "DATABASE_URL or FANWAAVE_DATABASE_URL is required")
    })?;
    let enqueue_secret = config.contact_enqueue_secret.clone().filter(|value| !value.trim().is_empty()).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "FANWAAVE_CONTACT_ENQUEUE_SECRET is required")
    })?;

    let db = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await?;

    // Fail closed if the canonical lib-core migration has not been applied.
    let schema_ready: Option<String> = sqlx::query_scalar(
        "SELECT to_regclass('public.contact_jobs')::text",
    )
    .fetch_one(&db)
    .await?;
    if schema_ready.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "fanwaave-0002 contact schema is missing; apply fanwaave-lib-core migrations",
        )
        .into());
    }

    let nats = match config.nats_url.as_deref() {
        Some(url) => match async_nats::connect(url).await {
            Ok(client) => Some(client),
            Err(error) => {
                warn!(%error, "NATS unavailable; contact jobs remain durable and workers will poll Postgres");
                None
            }
        },
        None => None,
    };

    let state = AppState::new(db, nats, enqueue_secret);
    let app = Router::new()
        .route("/healthz", get(|| async { Json(routes::health::body()) }))
        .route("/readyz", get(ready))
        .route("/v1/contact/jobs", post(routes::contact::enqueue))
        .route(
            "/v1/contact/jobs/{id}",
            get(routes::contact::get_job).delete(routes::contact::cancel_job),
        )
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&config.bind).await?;
    info!(bind=%config.bind, "fanwaave API listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn ready(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    match sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.db).await {
        Ok(_) => (StatusCode::OK, Json(json!({"ok": true, "database": "ready"}))),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"ok": false, "database": "unavailable"}))),
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

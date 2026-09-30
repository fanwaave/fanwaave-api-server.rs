#![forbid(unsafe_code)]

use std::time::Duration;

use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::{info, warn};

use crate::config::ApiConfig;
use crate::error::ServerError;
use crate::routes;
use crate::state::AppState;

pub async fn run(config: &ApiConfig) -> Result<(), ServerError> {
    let db = connect_database(config).await?;
    ensure_contact_schema(&db).await?;
    let nats = connect_nats(config).await;
    let enqueue_secret = required_enqueue_secret(config)?;
    let app = build_router(AppState::new(db, nats, enqueue_secret));
    let listener = tokio::net::TcpListener::bind(&config.bind)
        .await
        .map_err(ServerError::Io)?;
    info!(bind=%config.bind, "fanwaave API listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ServerError::Io)
}

async fn connect_database(config: &ApiConfig) -> Result<PgPool, ServerError> {
    let database_url = config
        .database_url
        .as_deref()
        .ok_or(ServerError::Configuration(
            "DATABASE_URL or FANWAAVE_DATABASE_URL is required",
        ))?;
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await
        .map_err(ServerError::Database)
}

async fn ensure_contact_schema(db: &PgPool) -> Result<(), ServerError> {
    let schema_ready: Option<String> = sqlx::query_scalar(
        "SELECT to_regclass('public.contact_jobs')::text",
    )
    .fetch_one(db)
    .await
    .map_err(ServerError::Database)?;
    if schema_ready.is_some() {
        Ok(())
    } else {
        Err(ServerError::Configuration(
            "fanwaave-0002 contact schema is missing; apply fanwaave-lib-core migrations",
        ))
    }
}

fn required_enqueue_secret(config: &ApiConfig) -> Result<String, ServerError> {
    config
        .contact_enqueue_secret
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or(ServerError::Configuration(
            "FANWAAVE_CONTACT_ENQUEUE_SECRET is required",
        ))
}

async fn connect_nats(config: &ApiConfig) -> Option<async_nats::Client> {
    match config.nats_url.as_deref() {
        Some(url) => match async_nats::connect(url).await {
            Ok(client) => Some(client),
            Err(error) => {
                warn!(%error, "NATS unavailable; contact jobs remain durable and workers will poll Postgres");
                None
            }
        },
        None => None,
    }
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { Json(routes::health::body()) }))
        .route("/readyz", get(ready))
        .route("/v1/contact/jobs", post(routes::contact::enqueue))
        .route(
            "/v1/contact/jobs/{id}",
            get(routes::contact::get_job).delete(routes::contact::cancel_job),
        )
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .with_state(state)
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

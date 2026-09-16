#![forbid(unsafe_code)]

use std::sync::Arc;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub nats: Option<async_nats::Client>,
    pub contact_enqueue_secret: Arc<str>,
}

impl AppState {
    pub fn new(db: PgPool, nats: Option<async_nats::Client>, contact_enqueue_secret: String) -> Self {
        Self {
            db,
            nats,
            contact_enqueue_secret: Arc::from(contact_enqueue_secret),
        }
    }
}

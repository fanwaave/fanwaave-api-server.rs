#![forbid(unsafe_code)]

use axum::{
    extract::{Path, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use fanwaave_lib_core::contact::{
    ContactChannel, ContactJobStatus, ContactJobView, ContactPayload, ContactProvider,
    EnqueueContactJob, EnqueueContactReceipt,
};
use serde::Serialize;
use serde_json::{json, Value};
use subtle::ConstantTimeEq;
use tracing::{error, warn};
use uuid::Uuid;

use crate::auth::require_bearer;
use crate::state::AppState;

const WAKE_SUBJECT: &str = "fanwaave.contact.wakeup";

pub async fn enqueue(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<EnqueueContactJob>,
) -> Result<(StatusCode, Json<EnqueueContactReceipt>), ApiHttpError> {
    authorize(&state, &headers)?;
    validate_enqueue(&request)?;

    if let Some(existing) = find_by_idempotency(&state, &request.idempotency_key).await? {
        return Ok((StatusCode::OK, Json(EnqueueContactReceipt {
            job_id: existing.0.to_string(),
            idempotency_key: request.idempotency_key,
            status: parse_status(&existing.1)?,
            duplicate: true,
        })));
    }

    let channel = request.contact.channel();
    let provider = channel.provider();
    let payload = payload_value(&request.contact)?;
    let campaign_id = request
        .campaign_id
        .as_deref()
        .map(Uuid::parse_str)
        .transpose()
        .map_err(|_| ApiHttpError::bad_request("campaign_id must be a UUID"))?;
    let metadata = if request.metadata.is_null() { json!({}) } else { request.metadata };

    let inserted = sqlx::query_as::<_, (Uuid, String)>(
        r#"
        INSERT INTO contact_jobs (
            idempotency_key, campaign_id, channel, provider, payload, metadata, priority, max_attempts
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        ON CONFLICT (idempotency_key) DO NOTHING
        RETURNING id, status
        "#,
    )
    .bind(&request.idempotency_key)
    .bind(campaign_id)
    .bind(channel.as_str())
    .bind(provider.as_str())
    .bind(payload)
    .bind(metadata)
    .bind(request.priority)
    .bind(request.max_attempts)
    .fetch_optional(&state.db)
    .await
    .map_err(ApiHttpError::database)?;

    let (id, status, duplicate) = match inserted {
        Some((id, status)) => (id, status, false),
        None => {
            let existing = find_by_idempotency(&state, &request.idempotency_key)
                .await?
                .ok_or_else(|| ApiHttpError::internal("idempotency conflict could not be resolved"))?;
            (existing.0, existing.1, true)
        }
    };

    if !duplicate {
        if let Some(client) = state.nats.as_ref() {
            let payload = serde_json::to_vec(&json!({"job_id": id, "channel": channel.as_str()}))
                .map_err(|_| ApiHttpError::internal("cannot serialize queue wakeup"))?;
            if let Err(error) = client.publish(WAKE_SUBJECT, payload.into()).await {
                warn!(job_id=%id, %error, "contact wakeup publish failed; worker polling remains authoritative");
            }
        }
    }

    let response_status = if duplicate { StatusCode::OK } else { StatusCode::ACCEPTED };
    Ok((response_status, Json(EnqueueContactReceipt {
        job_id: id.to_string(),
        idempotency_key: request.idempotency_key,
        status: parse_status(&status)?,
        duplicate,
    })))
}

pub async fn get_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<ContactJobView>, ApiHttpError> {
    authorize(&state, &headers)?;
    let row = sqlx::query_as::<_, (Uuid, String, String, String, String, i32, i32, Option<String>, Option<String>)>(
        r#"
        SELECT id, idempotency_key, channel, provider, status, attempt_count, max_attempts,
               provider_message_id, last_error
        FROM contact_jobs
        WHERE id=$1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(ApiHttpError::database)?
    .ok_or(ApiHttpError::NotFound)?;

    Ok(Json(ContactJobView {
        job_id: row.0.to_string(),
        idempotency_key: row.1,
        channel: parse_channel(&row.2)?,
        provider: parse_provider(&row.3)?,
        status: parse_status(&row.4)?,
        attempt_count: row.5,
        max_attempts: row.6,
        provider_message_id: row.7,
        last_error: row.8,
    }))
}

pub async fn cancel_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<ContactJobView>, ApiHttpError> {
    authorize(&state, &headers)?;
    let row = sqlx::query_as::<_, (Uuid, String, String, String, String, i32, i32, Option<String>, Option<String>)>(
        r#"
        UPDATE contact_jobs
        SET status='cancelled', lease_owner=NULL, lease_expires_at=NULL, updated_at=now()
        WHERE id=$1 AND status IN ('queued','retry')
        RETURNING id, idempotency_key, channel, provider, status, attempt_count, max_attempts,
                  provider_message_id, last_error
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(ApiHttpError::database)?
    .ok_or(ApiHttpError::Conflict("job is missing, leased, or already terminal"))?;

    Ok(Json(ContactJobView {
        job_id: row.0.to_string(),
        idempotency_key: row.1,
        channel: parse_channel(&row.2)?,
        provider: parse_provider(&row.3)?,
        status: parse_status(&row.4)?,
        attempt_count: row.5,
        max_attempts: row.6,
        provider_message_id: row.7,
        last_error: row.8,
    }))
}

async fn find_by_idempotency(state: &AppState, key: &str) -> Result<Option<(Uuid, String)>, ApiHttpError> {
    sqlx::query_as::<_, (Uuid, String)>("SELECT id, status FROM contact_jobs WHERE idempotency_key=$1")
        .bind(key)
        .fetch_optional(&state.db)
        .await
        .map_err(ApiHttpError::database)
}

fn authorize(state: &AppState, headers: &HeaderMap) -> Result<(), ApiHttpError> {
    let header = headers.get(AUTHORIZATION).and_then(|value| value.to_str().ok());
    let token = require_bearer(header).map_err(|_| ApiHttpError::Unauthorized)?;
    let expected = state.contact_enqueue_secret.as_bytes();
    let supplied = token.as_bytes();
    if supplied.len() != expected.len() || !bool::from(supplied.ct_eq(expected)) {
        return Err(ApiHttpError::Unauthorized);
    }
    Ok(())
}

fn validate_enqueue(request: &EnqueueContactJob) -> Result<(), ApiHttpError> {
    let key_len = request.idempotency_key.len();
    if !(8..=512).contains(&key_len) {
        return Err(ApiHttpError::bad_request("idempotency_key length must be 8..=512"));
    }
    if !(-1000..=1000).contains(&request.priority) {
        return Err(ApiHttpError::bad_request("priority must be -1000..=1000"));
    }
    if !(1..=20).contains(&request.max_attempts) {
        return Err(ApiHttpError::bad_request("max_attempts must be 1..=20"));
    }
    if !(request.metadata.is_null() || request.metadata.is_object()) {
        return Err(ApiHttpError::bad_request("metadata must be an object"));
    }
    match &request.contact {
        ContactPayload::Email(payload) => {
            if payload.to.trim().is_empty() || payload.subject.trim().is_empty() || payload.html.trim().is_empty() {
                return Err(ApiHttpError::bad_request("email to/subject/html must be non-empty"));
            }
            if payload.categories.len() > 10 {
                return Err(ApiHttpError::bad_request("email categories may contain at most 10 values"));
            }
        }
        ContactPayload::Sms(payload) => {
            if !payload.to.starts_with('+') || payload.body.trim().is_empty() {
                return Err(ApiHttpError::bad_request("sms requires E.164-like `to` and non-empty body"));
            }
        }
    }
    Ok(())
}

fn payload_value(contact: &ContactPayload) -> Result<Value, ApiHttpError> {
    match contact {
        ContactPayload::Email(payload) => serde_json::to_value(payload),
        ContactPayload::Sms(payload) => serde_json::to_value(payload),
    }
    .map_err(|_| ApiHttpError::internal("cannot serialize contact payload"))
}

fn parse_channel(value: &str) -> Result<ContactChannel, ApiHttpError> {
    if value == "email" {
        Ok(ContactChannel::Email)
    } else if value == "sms" {
        Ok(ContactChannel::Sms)
    } else {
        Err(ApiHttpError::internal("database contains unknown contact channel"))
    }
}

fn parse_provider(value: &str) -> Result<ContactProvider, ApiHttpError> {
    if value == "sendgrid" {
        Ok(ContactProvider::Sendgrid)
    } else if value == "twilio" {
        Ok(ContactProvider::Twilio)
    } else {
        Err(ApiHttpError::internal("database contains unknown contact provider"))
    }
}

fn parse_status(value: &str) -> Result<ContactJobStatus, ApiHttpError> {
    if value == "queued" {
        Ok(ContactJobStatus::Queued)
    } else if value == "leased" {
        Ok(ContactJobStatus::Leased)
    } else if value == "retry" {
        Ok(ContactJobStatus::Retry)
    } else if value == "sent" {
        Ok(ContactJobStatus::Sent)
    } else if value == "dead" {
        Ok(ContactJobStatus::Dead)
    } else if value == "cancelled" {
        Ok(ContactJobStatus::Cancelled)
    } else {
        Err(ApiHttpError::internal("database contains unknown contact job status"))
    }
}

#[derive(Debug)]
pub enum ApiHttpError {
    Unauthorized,
    NotFound,
    Conflict(&'static str),
    BadRequest(&'static str),
    Internal(&'static str),
}

impl ApiHttpError {
    fn bad_request(message: &'static str) -> Self {
        Self::BadRequest(message)
    }

    fn internal(message: &'static str) -> Self {
        Self::Internal(message)
    }

    fn database(error: sqlx::Error) -> Self {
        error!(%error, "contact queue database operation failed");
        Self::Internal("database operation failed")
    }
}

#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: &'static str,
}

impl IntoResponse for ApiHttpError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "valid bearer token required"),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found", "contact job not found"),
            Self::Conflict(message) => (StatusCode::CONFLICT, "conflict", message),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, "internal", message),
        };
        (status, Json(ErrorBody { error: error_code, message })).into_response()
    }
}

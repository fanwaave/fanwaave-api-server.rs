#![forbid(unsafe_code)]

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get as axum_get,
    Json, Router,
};
use ores_api_docs::{
    NoSection, OperationContext, OperationRequestData, RpcPayloadCodec, TypedOperationContext,
};
use ores_api_docs_operation_macros::ores_route;

use crate::state::AppState;

use super::handlers::{self, HealthGetOperation};

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/healthz", axum_get(get))
}

#[ores_route(operation = handlers::get_health)]
pub async fn get(State(state): State<AppState>) -> Response {
    let request = OperationRequestData::new(RpcPayloadCodec::Json);
    request.insert_path::<HealthGetOperation>(NoSection);
    request.insert_query::<HealthGetOperation>(NoSection);
    request.insert_headers::<HealthGetOperation>(NoSection);
    request.insert_body::<HealthGetOperation>(NoSection);
    let base = OperationContext::http(state);
    let context = TypedOperationContext::<AppState, HealthGetOperation>::new(base, request);

    match handlers::__ores_invoke_get_health(context).await {
        Ok(body) => Json(body).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

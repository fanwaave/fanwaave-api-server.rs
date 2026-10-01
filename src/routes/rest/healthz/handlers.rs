#![forbid(unsafe_code)]

use fanwaave_interfaces::Health;
use ores_api_docs::{NoSection, OperationSpec, RpcPayloadCodec, TypedOperationContext};
use ores_api_docs_operation_macros::ores_operation;

use crate::state::AppState;

pub struct HealthGetOperation;

impl OperationSpec for HealthGetOperation {
    type Path = NoSection;
    type Query = NoSection;
    type RequestHeaders = NoSection;
    type RequestBody = NoSection;
    type ResponseBody = Health;
    type ResponseHeaders = NoSection;
    type ResponseTrailers = NoSection;
    type Error = ();

    const KEY: &'static str = "fanwaave.health.get";
    const CODECS: &'static [RpcPayloadCodec] = &[RpcPayloadCodec::Json];
    const DEFAULT_CODEC: RpcPayloadCodec = RpcPayloadCodec::Json;
}

#[ores_operation(
    spec = HealthGetOperation,
    key = "fanwaave.health.get",
    codecs("json"),
    default_codec = "json",
    audiences("browser", "server"),
    scope = "regular"
)]
pub async fn get_health(
    _ctx: TypedOperationContext<AppState, HealthGetOperation>,
) -> Result<Health, ()> {
    Ok(crate::routes::health::body())
}

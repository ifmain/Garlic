use crate::{
    accounts::now,
    http::{bearer, blocking, header, Failure, Shared},
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use garlic_relay_data::control::{ErrorCode, ProxyError, Role, StreamOpen, User};
use serde_json::json;
use uuid::Uuid;

pub async fn user(app: Shared, headers: &HeaderMap) -> Result<User, Failure> {
    let token = bearer(headers)?.to_owned();
    let device = header(headers, "x-garlic-device-token")
        .unwrap_or("")
        .to_owned();
    let local = app.clone();
    let auth = token.clone();
    let (user, _) = blocking(move || local.accounts.user(&auth)).await?;
    crate::http::check_lease(app, &token, &user, &device).await?;
    Ok(user)
}
fn reject(error: ProxyError) -> Response {
    use ErrorCode::*;
    let status = match error.code {
        Unauthorized => StatusCode::UNAUTHORIZED,
        Forbidden => StatusCode::FORBIDDEN,
        SubscriptionExpired => StatusCode::PAYMENT_REQUIRED,
        StaleAuthority => StatusCode::CONFLICT,
        RouteUnavailable | CapacityExhausted => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::BAD_REQUEST,
    };
    let mut response = (status, Json(json!({"control_version":1,"error":error}))).into_response();
    response.extensions_mut().insert(error.code);
    response
}
pub async fn open(
    State(app): State<Shared>,
    headers: HeaderMap,
    Json(open): Json<StreamOpen>,
) -> Response {
    let who = match user(app.clone(), &headers).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };
    if let Err(error) = open.validate(&app.accounts.owner, now() * 1000) {
        return reject(error);
    }
    if open.user_id != who.id {
        return reject(ErrorCode::Forbidden.into());
    }
    let reply = json!({"control_version":1,"data_version":1,"flow_id":open.flow_id,"stream_id":open.stream_id,
        "accepted_capabilities":open.capabilities,"window_size":open.window_size,"deadline_unix_ms":open.deadline_unix_ms,"authority":app.accounts.owner});
    match app.hub.open(open, who.role == Role::Admin).await {
        Ok(()) => ([("x-garlic-relay-transport", "relay-shaped-v2")], Json(reply)).into_response(),
        Err(e) => reject(e),
    }
}
pub async fn close(
    State(app): State<Shared>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let who = match user(app.clone(), &headers).await {
        Ok(u) => u,
        Err(e) => return e.into_response(),
    };
    match app.hub.close(id, who.id) {
        Ok(()) => Json(json!({"ok":true})).into_response(),
        Err(e) => reject(e.into()),
    }
}

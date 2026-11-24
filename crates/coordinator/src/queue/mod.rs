pub mod executor;
pub mod model;
pub mod store;
pub mod transfers;
use crate::http::{bearer, blocking, Failure, Shared};
use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use garlic_relay_data::control::Role;
use model::*;
use serde_json::Value;
use uuid::Uuid;

pub fn routes() -> Router<Shared> {
    Router::new()
        .route("/tasks", post(submit))
        .route("/tasks/{id}", get(task))
        .route("/v1/infinity/tasks", post(submit))
        .route("/v1/infinity/tasks/{id}", get(task))
        .route("/v1/infinity/channels", post(channel))
        .route("/v1/infinity/channels/{id}/ready", post(ready))
        .route("/v1/infinity/channels/{id}/drain", post(drain))
        .route("/v1/infinity/channels/{id}/claim", post(claim))
        .route("/v1/infinity/channels/{id}/renew", post(renew))
        .route("/v1/infinity/channels/{id}/complete", post(complete))
        .route("/v1/infinity/channels/{id}/cancel", post(cancel))
        .route("/v1/infinity/capacity", post(capacity))
        .route("/v1/infinity/capacity/{id}", delete(release))
        .route("/v1/infinity/stats", get(stats))
        .route("/v1/infinity/nodes", get(nodes))
        .route("/v1/infinity/events", get(events))
        .layer(DefaultBodyLimit::max(2 << 20))
}
pub(crate) async fn node(
    app: &Shared,
    headers: &HeaderMap,
) -> std::result::Result<String, Failure> {
    let token = bearer(headers)?.to_owned();
    let app = app.clone();
    blocking(move || crate::network::identity(&app, &token)).await
}
async fn admin(app: &Shared, h: &HeaderMap) -> std::result::Result<(), Failure> {
    let token = bearer(h)?.to_owned();
    let app = app.clone();
    blocking(move || {
        app.accounts.require_admin(&token)?;
        Ok(())
    })
    .await
}
type JsonReply = std::result::Result<Json<Value>, Failure>;
async fn submit(
    State(app): State<Shared>,
    h: HeaderMap,
    Json(body): Json<Task>,
) -> std::result::Result<Response, Failure> {
    let u = crate::streams::user(app.clone(), &h).await?;
    let (new, r) = blocking(move || app.jobs.with(|s, p, _| s.submit(p, body, u.id))).await?;
    Ok((
        if new {
            StatusCode::ACCEPTED
        } else {
            StatusCode::OK
        },
        Json(r),
    )
        .into_response())
}
async fn task(
    State(app): State<Shared>,
    Path(id): Path<Uuid>,
    h: HeaderMap,
) -> std::result::Result<Json<Record>, Failure> {
    let u = crate::streams::user(app.clone(), &h).await?;
    Ok(Json(
        blocking(move || {
            app.jobs
                .with(|s, _, _| s.get(id, u.id, u.role == Role::Admin))
        })
        .await?,
    ))
}
async fn channel(
    State(app): State<Shared>,
    h: HeaderMap,
    Json(body): Json<Registration>,
) -> JsonReply {
    let n = node(&app, &h).await?;
    Ok(Json(
        blocking(move || app.jobs.with(|s, _, now| s.open(body, &n, now))).await?,
    ))
}
async fn ready(
    State(app): State<Shared>,
    Path(id): Path<Uuid>,
    h: HeaderMap,
    Json(body): Json<WorkerRequest>,
) -> JsonReply {
    let n = node(&app, &h).await?;
    Ok(Json(
        blocking(move || app.jobs.with(|s, _, now| s.ready(id, body, &n, now))).await?,
    ))
}
async fn drain(
    State(app): State<Shared>,
    Path(id): Path<Uuid>,
    h: HeaderMap,
    Json(body): Json<WorkerRequest>,
) -> JsonReply {
    let n = node(&app, &h).await?;
    Ok(Json(
        blocking(move || app.jobs.with(|s, _, now| s.drain(id, body, &n, now))).await?,
    ))
}
async fn claim(
    State(app): State<Shared>,
    Path(id): Path<Uuid>,
    h: HeaderMap,
    Json(body): Json<WorkerRequest>,
) -> JsonReply {
    let n = node(&app, &h).await?;
    Ok(Json(
        blocking(move || app.jobs.with(|s, p, now| s.claim(p, id, body, &n, now))).await?,
    ))
}
async fn renew(
    State(app): State<Shared>,
    Path(id): Path<Uuid>,
    h: HeaderMap,
    Json(body): Json<LeaseRequest>,
) -> JsonReply {
    let n = node(&app, &h).await?;
    Ok(Json(
        blocking(move || app.jobs.with(|s, p, now| s.renew(p, id, body, &n, now))).await?,
    ))
}
async fn complete(
    State(app): State<Shared>,
    Path(id): Path<Uuid>,
    h: HeaderMap,
    Json(body): Json<Completion>,
) -> std::result::Result<(), Failure> {
    let n = node(&app, &h).await?;
    blocking(move || app.jobs.with(|s, p, now| s.complete(p, id, body, &n, now))).await
}
async fn cancel(
    State(app): State<Shared>,
    Path(id): Path<Uuid>,
    h: HeaderMap,
    Json(body): Json<LeaseRequest>,
) -> std::result::Result<(), Failure> {
    let n = node(&app, &h).await?;
    blocking(move || app.jobs.with(|s, p, now| s.cancel(p, id, body, &n, now))).await
}
async fn capacity(State(app): State<Shared>, h: HeaderMap, Json(body): Json<Reserve>) -> JsonReply {
    let u = crate::streams::user(app.clone(), &h).await?;
    Ok(Json(
        blocking(move || app.jobs.with(|s, _, now| s.capacity(body, u.id, now))).await?,
    ))
}
async fn release(
    State(app): State<Shared>,
    Path(id): Path<Uuid>,
    h: HeaderMap,
) -> std::result::Result<StatusCode, Failure> {
    let u = crate::streams::user(app.clone(), &h).await?;
    blocking(move || app.jobs.with(|s, _, _| s.release(id, u.id))).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn stats(State(app): State<Shared>, h: HeaderMap) -> JsonReply {
    let token = bearer(&h)?.to_owned();
    Ok(Json(
        blocking(move || {
            if app.accounts.user(&token).is_err() {
                crate::network::identity(&app, &token)?;
            }
            app.jobs.with(|s, _, _| Ok(s.stats(&app.accounts.owner)))
        })
        .await?,
    ))
}
async fn nodes(State(app): State<Shared>, h: HeaderMap) -> JsonReply {
    admin(&app, &h).await?;
    Ok(Json(
        blocking(move || app.jobs.with(|s, _, now| Ok(s.nodes(now)))).await?,
    ))
}
async fn events(State(app): State<Shared>, h: HeaderMap) -> JsonReply {
    admin(&app, &h).await?;
    Ok(Json(
        blocking(move || app.jobs.with(|s, _, _| Ok(s.events()))).await?,
    ))
}

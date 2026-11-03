//! Garlic DNS: priority queue, leases, idempotency and durable snapshots.
use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, routing::{get, post}, Json, Router};
use garlic_core::{ClaimRequest, ClaimResponse, CompleteRequest, Health, Priority, TaskRequest, TaskStatus, TaskView};
use serde::{Deserialize, Serialize};
use std::{collections::{HashMap, VecDeque}, env, path::PathBuf, sync::Arc, time::{Duration, Instant}};
use tokio::sync::Mutex;
use uuid::Uuid;

const MAX_PENDING: usize = 10_000;
const LEASE_SECONDS: u64 = 30;
type ApiResult<T> = Result<T, StatusCode>;
#[derive(Clone)]
struct AppState { queue: Arc<Mutex<Queue>>, file: PathBuf, token: Option<String> }
struct Record { view: TaskView, lease: Option<(String, Uuid, Instant)> }
#[derive(Default)]
struct Queue { records: HashMap<Uuid, Record>, high: VecDeque<Uuid>, normal: VecDeque<Uuid>, low: VecDeque<Uuid> }
#[derive(Serialize, Deserialize)]
struct Snapshot { records: Vec<TaskView> }
impl Queue {
    fn push(&mut self, id: Uuid, priority: Priority) {
        match priority { Priority::High => &mut self.high, Priority::Normal => &mut self.normal, Priority::Low => &mut self.low }.push_back(id);
    }
    fn pop(&mut self) -> Option<Uuid> { self.high.pop_front().or_else(|| self.normal.pop_front()).or_else(|| self.low.pop_front()) }
    fn pending_len(&self) -> usize { self.high.len() + self.normal.len() + self.low.len() }
    fn snapshot(&self) -> Snapshot { Snapshot { records: self.records.values().map(|r| r.view.clone()).collect() } }
    fn restore(file: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if !file.exists() { return Ok(Self::default()); }
        let snapshot: Snapshot = serde_json::from_slice(&std::fs::read(file)?)?;
        let mut queue = Self::default();
        for mut view in snapshot.records {
            if view.status == TaskStatus::Leased {
                if view.task.exchange.as_ref().is_some_and(|x| !matches!(x.method.to_ascii_uppercase().as_str(), "GET" | "HEAD")) {
                    view.status = TaskStatus::Failed;
                    view.error = Some("outcome_unknown: coordinator restarted; mutation replay blocked".into());
                } else { view.status = TaskStatus::Pending; }
            }
            let id = view.task.task_id;
            if view.status == TaskStatus::Pending { queue.push(id, view.task.priority); }
            queue.records.insert(id, Record { view, lease: None });
        }
        Ok(queue)
    }
}
fn persist(state: &AppState, q: &Queue) -> std::io::Result<()> {
    if let Some(parent) = state.file.parent() { std::fs::create_dir_all(parent)?; }
    let tmp = state.file.with_extension("tmp");
    let data = serde_json::to_vec(&q.snapshot()).map_err(std::io::Error::other)?;
    std::fs::write(&tmp, data)?;
    std::fs::rename(tmp, &state.file)?;
    Ok(())
}
fn authorize(headers: &HeaderMap, token: &Option<String>) -> ApiResult<()> {
    if let Some(required) = token {
        if headers.get("x-garlic-key").and_then(|h| h.to_str().ok()) != Some(required.as_str()) { return Err(StatusCode::UNAUTHORIZED); }
    }
    Ok(())
}
async fn health() -> Json<Health> { Json(Health { service: "garlic-dns".into(), healthy: true }) }
async fn submit(State(s): State<AppState>, headers: HeaderMap, Json(task): Json<TaskRequest>) -> ApiResult<(StatusCode, Json<TaskView>)> {
    authorize(&headers, &s.token)?;
    if serde_json::to_vec(&task).map_err(|_| StatusCode::BAD_REQUEST)?.len() > 2_000_000 { return Err(StatusCode::PAYLOAD_TOO_LARGE); }
    let mut q = s.queue.lock().await;
    if let Some(r) = q.records.get(&task.task_id) {
        if r.view.task != task { return Err(StatusCode::CONFLICT); }
        return Ok((StatusCode::OK, Json(r.view.clone())));
    }
    if q.pending_len() >= MAX_PENDING { return Err(StatusCode::SERVICE_UNAVAILABLE); }
    let view = TaskView::pending(task.clone());
    q.records.insert(task.task_id, Record { view: view.clone(), lease: None });
    q.push(task.task_id, task.priority);
    persist(&s, &q).map_err(|e| { eprintln!("persist failed: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok((StatusCode::ACCEPTED, Json(view)))
}
async fn get_task(Path(id): Path<Uuid>, State(s): State<AppState>, headers: HeaderMap) -> ApiResult<Json<TaskView>> {
    authorize(&headers, &s.token)?;
    s.queue.lock().await.records.get(&id).map(|r| Json(r.view.clone())).ok_or(StatusCode::NOT_FOUND)
}
async fn claim(State(s): State<AppState>, headers: HeaderMap, Json(request): Json<ClaimRequest>) -> ApiResult<Json<ClaimResponse>> {
    authorize(&headers, &s.token)?;
    if request.worker_id.is_empty() || request.worker_id.len() > 128 { return Err(StatusCode::BAD_REQUEST); }
    let mut q = s.queue.lock().await;
    while let Some(id) = q.pop() {
        if let Some(rec) = q.records.get_mut(&id) {
            if rec.view.status != TaskStatus::Pending { continue; }
            let lease_id = Uuid::new_v4();
            rec.view.status = TaskStatus::Leased;
            rec.view.attempts += 1;
            rec.lease = Some((request.worker_id.clone(), lease_id, Instant::now() + Duration::from_secs(LEASE_SECONDS)));
            let task = rec.view.task.clone();
            persist(&s, &q).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            return Ok(Json(ClaimResponse { task: Some(task), lease_id: Some(lease_id) }));
        }
    }
    Ok(Json(ClaimResponse { task: None, lease_id: None }))
}
async fn complete(State(s): State<AppState>, headers: HeaderMap, Json(request): Json<CompleteRequest>) -> ApiResult<StatusCode> {
    authorize(&headers, &s.token)?;
    let mut q = s.queue.lock().await;
    let rec = q.records.get_mut(&request.task_id).ok_or(StatusCode::NOT_FOUND)?;
    if rec.view.status == TaskStatus::Completed || rec.view.status == TaskStatus::Failed {
        return if rec.lease.as_ref().is_some_and(|(owner, lease, _)| owner == &request.worker_id && lease == &request.lease_id)
            && request.result.task == rec.view.task && request.result.status == rec.view.status && request.result.result == rec.view.result && request.result.exchange == rec.view.exchange && request.result.error == rec.view.error {
            Ok(StatusCode::OK)
        } else { Err(StatusCode::CONFLICT) };
    }
    if rec.view.status != TaskStatus::Leased || rec.lease.as_ref().is_none_or(|(owner, lease, until)| owner != &request.worker_id || lease != &request.lease_id || Instant::now() > *until) {
        return Err(StatusCode::CONFLICT);
    }
    if request.result.task != rec.view.task || !matches!(request.result.status, TaskStatus::Completed | TaskStatus::Failed) { return Err(StatusCode::BAD_REQUEST); }
    rec.view.status = request.result.status;
    rec.view.result = request.result.result;
    rec.view.exchange = request.result.exchange;
    rec.view.error = request.result.error;
    persist(&s, &q).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}
async fn reaper(s: AppState) {
    let mut tick = tokio::time::interval(Duration::from_secs(2));
    loop {
        tick.tick().await;
        let mut q = s.queue.lock().await;
        let expired: Vec<_> = q.records.iter_mut().filter_map(|(id, r)| {
            if r.view.status == TaskStatus::Leased && r.lease.as_ref().is_some_and(|(_, _, until)| Instant::now() > *until) {
                if r.view.task.exchange.as_ref().is_some_and(|x| !matches!(x.method.to_ascii_uppercase().as_str(), "GET" | "HEAD")) {
                    r.view.status = TaskStatus::Failed;
                    r.view.error = Some("outcome_unknown: lease expired; automatic mutation replay blocked".into());
                } else { r.view.status = TaskStatus::Pending; }
                r.lease = None;
                Some((*id, r.view.task.priority))
            } else { None }
        }).collect();
        if !expired.is_empty() {
            for (id, p) in expired { q.push(id, p); }
            if let Err(e) = persist(&s, &q) { eprintln!("reaper persistence error: {e}"); }
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = env::var("GARLIC_DNS_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let file = PathBuf::from(env::var("GARLIC_STATE_FILE").unwrap_or_else(|_| "garlic-state.json".into()));
    let token = env::var("GARLIC_API_KEY").ok().filter(|v| !v.is_empty());
    if !address.parse::<std::net::SocketAddr>()?.ip().is_loopback()
        && env::var("GARLIC_DEV_EXTERNAL_BIND").as_deref() != Ok("1") {
        return Err("External plaintext bind blocked; use a TLS reverse proxy on loopback or explicitly set GARLIC_DEV_EXTERNAL_BIND=1 for development".into());
    }
    let queue = Queue::restore(&file)?;
    let state = AppState { queue: Arc::new(Mutex::new(queue)), file, token };
    tokio::spawn(reaper(state.clone()));
    let router = Router::new().route("/health", get(health)).route("/tasks", post(submit))
        .route("/tasks/{id}", get(get_task)).route("/workers/claim", post(claim))
        .route("/workers/complete", post(complete)).with_state(state);
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("garlic-dns listening at http://{address}");
    axum::serve(listener, router).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use garlic_core::{Operation, ExchangeRequest};
    #[test]
    fn restart_does_not_replay_mutation() {
        let file = std::env::temp_dir().join(format!("garlic-{}.json", Uuid::new_v4()));
        let task = TaskRequest { task_id: Uuid::new_v4(), operation: Operation::Http,
            input: String::new(), priority: Priority::Normal,
            exchange: Some(ExchangeRequest { service: "demo".into(), method: "POST".into(),
                path: "/".into(), headers: Default::default(), body_base64: String::new() }) };
        let mut view = TaskView::pending(task.clone());
        view.status = TaskStatus::Leased;
        std::fs::write(&file, serde_json::to_vec(&Snapshot { records: vec![view] }).unwrap()).unwrap();
        let q = Queue::restore(&file).unwrap();
        assert_eq!(q.records[&task.task_id].view.status, TaskStatus::Failed);
        assert_eq!(q.pending_len(), 0);
        std::fs::remove_file(file).unwrap();
    }
}

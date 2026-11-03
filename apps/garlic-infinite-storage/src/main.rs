//! Garlic Infinite Storage: direct and queue-based request/response exchange.
use axum::{extract::State, http::{HeaderMap, StatusCode}, routing::{get, post}, Json, Router};
use base64::{engine::general_purpose::STANDARD, Engine};
use garlic_core::{execute_text, safe_relative_path, ClaimRequest, ClaimResponse, CompleteRequest, ExchangeResponse, Health, Operation, TaskRequest, TaskStatus, TaskView, MAX_BODY_BYTES};
use std::{collections::BTreeMap, env, sync::Arc, time::Duration};

#[derive(Clone)]
struct StateData { name: String, services: Arc<BTreeMap<String, String>>, token: Option<String>, http: reqwest::Client }
fn authorize(headers: &HeaderMap, token: &Option<String>) -> Result<(), StatusCode> {
    if let Some(required) = token {
        if headers.get("x-garlic-key").and_then(|h| h.to_str().ok()) != Some(required.as_str()) { return Err(StatusCode::UNAUTHORIZED); }
    }
    Ok(())
}
async fn health(State(s): State<Arc<StateData>>) -> Json<Health> { Json(Health { service: s.name.clone(), healthy: true }) }
async fn execute(State(s): State<Arc<StateData>>, headers: HeaderMap, Json(task): Json<TaskRequest>) -> Result<Json<TaskView>, StatusCode> {
    authorize(&headers, &s.token)?;
    Ok(Json(run_task(&s, task).await))
}
async fn run_task(s: &StateData, task: TaskRequest) -> TaskView {
    if task.operation != Operation::Http { return execute_text(&task); }
    let mut view = TaskView::pending(task.clone());
    view.attempts = 1;
    let outcome: Result<ExchangeResponse, String> = async {
        let exchange = task.exchange.as_ref().ok_or("missing exchange specification")?;
        let base = s.services.get(&exchange.service).ok_or("service not configured on this node")?;
        if !safe_relative_path(&exchange.path) { return Err("invalid relative path".into()); }
        let method = reqwest::Method::from_bytes(exchange.method.as_bytes()).map_err(|_| "invalid method")?;
        if !matches!(method, reqwest::Method::GET | reqwest::Method::POST | reqwest::Method::PUT | reqwest::Method::PATCH | reqwest::Method::DELETE | reqwest::Method::HEAD) {
            return Err("method not allowed".into());
        }
        let bytes = STANDARD.decode(&exchange.body_base64).map_err(|_| "invalid base64 body")?;
        if bytes.len() > MAX_BODY_BYTES { return Err("request too large".into()); }
        let url = format!("{}{}", base.trim_end_matches('/'), exchange.path);
        let mut request = s.http.request(method, url).body(bytes);
        for (key, val) in &exchange.headers {
            if !matches!(key.to_ascii_lowercase().as_str(), "content-type" | "accept" | "accept-language" | "if-none-match" | "if-modified-since") { continue; }
            request = request.header(key.as_str(), val.as_str());
        }
        let mut response = request.send().await.map_err(|e| format!("upstream request failed: {e}"))?;
        let status = response.status().as_u16();
        let mut headers = BTreeMap::new();
        for key in ["content-type", "etag", "last-modified"] {
            if let Some(v) = response.headers().get(key).and_then(|h| h.to_str().ok()) {
                headers.insert(key.to_string(), v.to_string());
            }
        }
        if response.content_length().is_some_and(|n| n > MAX_BODY_BYTES as u64) { return Err("response too large".into()); }
        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
            if chunk.len() > MAX_BODY_BYTES - body.len() { return Err("response too large".into()); }
            body.extend_from_slice(&chunk);
        }
        Ok(ExchangeResponse { status, headers, body_base64: STANDARD.encode(body) })
    }.await;
    match outcome {
        Ok(result) => { view.status = TaskStatus::Completed; view.exchange = Some(result); }
        Err(error) => { view.status = TaskStatus::Failed; view.error = Some(error); }
    }
    view
}
async fn poll_loop(dns: String, worker: String, state: Arc<StateData>) {
    let mut backoff = Duration::from_millis(250);
    loop {
        let mut req = state.http.post(format!("{dns}/workers/claim")).json(&ClaimRequest { worker_id: worker.clone() });
        if let Some(key) = &state.token { req = req.header("x-garlic-key", key); }
        let claim = req.send().await;
        match claim {
            Ok(resp) if resp.status().is_success() => {
                backoff = Duration::from_millis(250);
                match resp.json::<ClaimResponse>().await {
                    Ok(ClaimResponse { task: Some(task), lease_id: Some(lease_id) }) => {
                        let result = run_task(&state, task.clone()).await;
                        let body = CompleteRequest { task_id: task.task_id, worker_id: worker.clone(), lease_id, result };
                        for _ in 0..3 {
                            let mut send = state.http.post(format!("{dns}/workers/complete")).json(&body);
                            if let Some(key) = &state.token { send = send.header("x-garlic-key", key); }
                            if let Ok(response) = send.send().await {
                                if response.status().is_success() || response.status() == StatusCode::CONFLICT { break; }
                            }
                            tokio::time::sleep(Duration::from_millis(250)).await;
                        }
                    }
                    Ok(_) => tokio::time::sleep(Duration::from_millis(250)).await,
                    Err(error) => eprintln!("claim decoding error: {error}"),
                }
            }
            _ => {
                eprintln!("{worker}: control unavailable");
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(5));
            }
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = env::var("GARLIC_STORAGE_BIND").unwrap_or_else(|_| "127.0.0.1:8090".into());
    let dns = env::var("GARLIC_DNS_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
    let name = env::var("GARLIC_STORAGE_NAME").unwrap_or_else(|_| "storage-a".into());
    let slots: usize = env::var("GARLIC_WORKERS").ok().and_then(|s| s.parse().ok()).unwrap_or(4).clamp(1, 40);
    let mut services = BTreeMap::new();
    // Example: GARLIC_SERVICES=demo=http://127.0.0.1:9000,public=https://example.org
    for item in env::var("GARLIC_SERVICES").unwrap_or_default().split(',') {
        if let Some((key, url)) = item.split_once('=') {
            if !key.is_empty() && (url.starts_with("https://") || url.starts_with("http://127.0.0.1:") || url.starts_with("http://localhost:")) {
                services.insert(key.to_string(), url.trim_end_matches('/').to_string());
            }
        }
    }
    let token = env::var("GARLIC_API_KEY").ok().filter(|x| !x.is_empty());
    if !address.parse::<std::net::SocketAddr>()?.ip().is_loopback()
        && env::var("GARLIC_DEV_EXTERNAL_BIND").as_deref() != Ok("1") {
        return Err("External plaintext bind blocked; use a TLS reverse proxy on loopback or explicitly set GARLIC_DEV_EXTERNAL_BIND=1 for development".into());
    }
    let http = reqwest::Client::builder().timeout(Duration::from_secs(12)).redirect(reqwest::redirect::Policy::none()).build()?;
    let state = Arc::new(StateData { name: name.clone(), services: Arc::new(services), token, http });
    for index in 0..slots {
        tokio::spawn(poll_loop(dns.clone(), format!("{name}-{index}"), state.clone()));
    }
    let app = Router::new().route("/health", get(health)).route("/execute", post(execute)).with_state(state);
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("garlic-infinite-storage listening at http://{address}, {slots} workers");
    axum::serve(listener, app).await?;
    Ok(())
}

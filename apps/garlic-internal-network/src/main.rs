//! Garlic Internal Network: Windows CLI, portable Rust request/response client.
use base64::{engine::general_purpose::STANDARD, Engine};
use garlic_core::{ExchangeRequest, Operation, Priority, TaskRequest, TaskStatus, TaskView};
use std::{collections::BTreeMap, env, time::{Duration, Instant}};
use uuid::Uuid;

fn endpoint_list(var: &str, default: &str) -> Vec<String> {
    env::var(var).unwrap_or_else(|_| default.into()).split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned).collect()
}
fn make_task(args: &[String]) -> Result<TaskRequest, String> {
    let operation = match args.first().map(String::as_str) {
        Some("echo") => Operation::Echo, Some("uppercase") => Operation::Uppercase,
        Some("reverse") => Operation::Reverse, Some("http") => Operation::Http,
        _ => return Err("Usage: <echo|uppercase|reverse> <text> [priority] OR http <service> <METHOD> <relative-path> [body]".into()),
    };
    let (input, exchange) = if operation == Operation::Http {
        if args.len() < 4 { return Err("Usage: http <service> <METHOD> <relative-path> [body]".into()); }
        (String::new(), Some(ExchangeRequest {
            service: args[1].clone(), method: args[2].clone(), path: args[3].clone(),
            headers: BTreeMap::new(), body_base64: STANDARD.encode(args.get(4).map(String::as_bytes).unwrap_or_default()),
        }))
    } else {
        (args.get(1).cloned().ok_or("missing text")?, None)
    };
    let priority = match args.get(2).map(String::as_str) {
        Some("high") if exchange.is_none() => Priority::High,
        Some("low") if exchange.is_none() => Priority::Low,
        _ => Priority::Normal,
    };
    Ok(TaskRequest { operation, input, exchange, priority, task_id: Uuid::new_v4() })
}
fn print_result(route: &str, view: TaskView) -> Result<(), String> {
    println!("route={route} task_id={} attempts={} status={:?}", view.task.task_id, view.attempts, view.status);
    if view.status == TaskStatus::Failed { return Err(view.error.unwrap_or_else(|| "task failed".into())); }
    if let Some(result) = view.result { println!("result={result}"); }
    if let Some(exchange) = view.exchange {
        let bytes = STANDARD.decode(exchange.body_base64).map_err(|e| e.to_string())?;
        println!("http_status={} headers={:?} body_bytes={}", exchange.status, exchange.headers, bytes.len());
        match String::from_utf8(bytes.clone()) {
            Ok(text) => println!("body={text}"),
            Err(_) => { std::fs::write("garlic-response.bin", &bytes).map_err(|e| e.to_string())?; println!("binary response saved to garlic-response.bin"); }
        }
    }
    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    let job = make_task(&args)?;
    let http = reqwest::Client::builder().timeout(Duration::from_secs(5)).build()?;
    let token = env::var("GARLIC_API_KEY").ok().filter(|s| !s.is_empty());
    // Mutating HTTP operations are sent through one coordinator, never retried
    // automatically across different direct nodes after an ambiguous timeout.
    let mutating_http = job.exchange.as_ref().is_some_and(|x| !matches!(x.method.to_ascii_uppercase().as_str(), "GET" | "HEAD"));
    if mutating_http { eprintln!("non-read-only operation: using one queued route; upstream must support idempotency for lease retries"); }
    if !mutating_http { for url in endpoint_list("GARLIC_STORAGE_URLS", "http://127.0.0.1:8090") {
        let start = Instant::now();
        let mut req = http.post(format!("{url}/execute")).json(&job);
        if let Some(key) = &token { req = req.header("x-garlic-key", key); }
        if let Ok(resp) = req.send().await {
            if resp.status().is_success() {
                if let Ok(view) = resp.json::<TaskView>().await {
                    println!("direct latency_ms={}", start.elapsed().as_millis());
                    return print_result(&url, view).map_err(Into::into);
                }
            }
        }
        eprintln!("node {url}: slow or unavailable; trying another");
    }}
    for control in endpoint_list("GARLIC_CONTROL_URLS", "http://127.0.0.1:8080") {
        let mut req = http.post(format!("{control}/tasks")).json(&job);
        if let Some(key) = &token { req = req.header("x-garlic-key", key); }
        let response = req.send().await.map_err(|_| format!("Unknown submission outcome for {} at {control}; inspect this coordinator before retrying", job.task_id))?;
        if !response.status().is_success() { return Err(format!("Coordinator {control} returned {} for {}; inspect status before retrying", response.status(), job.task_id).into()); }
        let deadline = Instant::now() + Duration::from_secs(45);
        while Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(300)).await;
            let mut check = http.get(format!("{control}/tasks/{}", job.task_id));
            if let Some(key) = &token { check = check.header("x-garlic-key", key); }
            if let Ok(response) = check.send().await {
                if let Ok(view) = response.json::<TaskView>().await {
                    if matches!(view.status, TaskStatus::Completed | TaskStatus::Failed) {
                        return print_result(&format!("infinity:{control}"), view).map_err(Into::into);
                    }
                }
            }
        }
        eprintln!("task accepted but still pending: {} at {control}. Do not blindly resubmit with a new ID.", job.task_id);
        return Err("task pending; deadline reached".into());
    }
    Err("no available Storage or Control API".into())
}

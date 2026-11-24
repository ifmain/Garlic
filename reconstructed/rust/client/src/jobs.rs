use crate::api::Api;
use crate::device::Devices;
use crate::settings::{origin, time, Config, SavedSession, Settings};
use base64::{engine::general_purpose::STANDARD, Engine};
use garlic_relay_data::control::Authority;
use garlic_state::StateError;
use reqwest::Method;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};
use uuid::Uuid;

const TASKS: &str = "/v1/infinity/tasks";

#[derive(Deserialize)]
struct Record {
    task: Identity,
    status: String,
    attempts: u32,
    result: Option<String>,
    exchange: Option<Response>,
    error: Option<String>,
}
#[derive(Deserialize)]
struct Identity {
    task_id: Uuid,
}
#[derive(Deserialize)]
struct Response {
    status: u16,
    headers: BTreeMap<String, String>,
    body_base64: String,
}

impl Record {
    fn read(value: Value, id: Uuid) -> Result<Self, StateError> {
        let record: Self = serde_json::from_value(value)?;
        if record.task.task_id != id {
            return Err("queue returned another task identifier".into());
        }
        Ok(record)
    }
    fn print(&self, route: &str) -> Result<(), StateError> {
        let state = match self.status.as_str() {
            "completed" => "Completed",
            "failed" => "Failed",
            "pending" => "Pending",
            "leased" => "Leased",
            _ => return Err("unknown queue status".into()),
        };
        println!(
            "route={route} task_id={} attempts={} status={state}",
            self.task.task_id, self.attempts
        );
        if state == "Failed" {
            return Err(self
                .error
                .as_deref()
                .unwrap_or("task failed")
                .to_string()
                .into());
        }
        if let Some(text) = &self.result {
            println!("result={text}");
        }
        if let Some(response) = &self.exchange {
            let body = STANDARD.decode(&response.body_base64)?;
            println!(
                "http_status={} headers={:?} body_bytes={}",
                response.status,
                response.headers,
                body.len()
            );
            if let Ok(text) = std::str::from_utf8(&body) {
                println!("body={text}");
            } else {
                std::fs::write("garlic-response.bin", body)?;
                println!("binary response saved to garlic-response.bin");
            }
        }
        Ok(())
    }
}

fn make_task(args: &[&str], id: Uuid) -> Result<Value, StateError> {
    let op = args[0];
    let (input, priority, exchange) = match args {
        ["http", service, method, path, body @ ..] if body.len() <= 1 => {
            let value = json!({"service":service, "method":method, "path":path,
                "headers":{}, "body_base64":STANDARD.encode(body.first().copied().unwrap_or(""))});
            ("", "normal", value)
        }
        ["echo" | "reverse" | "uppercase", text, priority @ ..] if priority.len() <= 1 => {
            let priority = match priority.first() {
                Some(&"low") => "low",
                Some(&"high") => "high",
                _ => "normal",
            };
            (*text, priority, Value::Null)
        }
        _ => return Err("expected <echo|uppercase|reverse> <text> [priority] or http <service> <METHOD> <relative-path> [body]".into()),
    };
    Ok(
        json!({"operation":op, "input":input, "priority":priority, "exchange":exchange, "task_id":id}),
    )
}

fn routes(var: &str, configured: &[String]) -> Result<Vec<String>, StateError> {
    let values = match std::env::var(var) {
        Ok(value) => value
            .split(',')
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
            .collect(),
        Err(_) => configured.to_vec(),
    };
    values.iter().map(|v| origin(v)).collect()
}

struct Queue<'a> {
    api: &'a Api,
    devices: &'a Devices,
    url: &'a str,
    session: &'a SavedSession,
    device: Option<String>,
    authority: Authority,
}
impl<'a> Queue<'a> {
    fn open(
        api: &'a Api,
        devices: &'a Devices,
        url: &'a str,
        session: &'a SavedSession,
    ) -> Result<Self, StateError> {
        let stats = api.call(
            url,
            Method::GET,
            "/v1/infinity/stats",
            Some(&session.token),
            None,
            None,
        )?;
        let authority: Authority = serde_json::from_value(stats["authority"].clone())?;
        let info = api.call(
            url,
            Method::GET,
            "/v1/auth/info",
            Some(&session.token),
            None,
            None,
        )?;
        let device = if info["device_leases"] == true {
            devices.claim(url, session, false)?
        } else {
            None
        };
        Ok(Self {
            api,
            devices,
            url,
            session,
            device,
            authority,
        })
    }
    fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value, StateError> {
        self.api.call(
            self.url,
            method,
            path,
            Some(&self.session.token),
            self.device.as_deref(),
            body,
        )
    }
    fn record(&self, id: Uuid) -> Result<Record, StateError> {
        Record::read(
            self.request(Method::GET, &format!("{TASKS}/{id}"), None)?,
            id,
        )
    }
}

fn execute(args: &[&str]) -> Result<(), StateError> {
    let settings = Settings::locate()?;
    let cfg = settings.load_effective()?;
    let api = Api::new(&cfg)?;
    let devices = Devices::new(api.clone(), &settings)?;
    let sessions = settings.sessions()?;
    if args[0] == "status" {
        let ["status", url, id, expected @ ..] = args else {
            return Err("expected status <owner-url> <task-id> [<owner-id> <epoch>]".into());
        };
        let url = origin(url)?;
        let id = Uuid::parse_str(id)?;
        let session = sessions
            .entries
            .get(&url)
            .filter(|s| s.expires_at > time())
            .ok_or("no active session for owner")?;
        let queue = Queue::open(&api, &devices, &url, session)?;
        match expected {
            [] => (),
            [owner, epoch]
                if Uuid::parse_str(owner)? == queue.authority.owner_id
                    && epoch.parse::<u64>()? == queue.authority.epoch =>
            {
                ()
            }
            _ => return Err("task authority or epoch mismatch".into()),
        }
        return queue.record(id)?.print(&url);
    }
    let id = Uuid::new_v4();
    let body = make_task(args, id)?;
    eprintln!("task_id={id}");
    let direct = routes("GARLIC_STORAGE_URLS", &cfg.storage_urls)?;
    let entries = routes("GARLIC_CONTROL_URLS", &cfg.control_urls)?;
    let mut seen = BTreeSet::new();
    let mut last_error: StateError = "no active session for a configured queue".into();
    for (url, queued) in direct
        .iter()
        .filter(|_| cfg.mode != "ru")
        .map(|url| (url, false))
        .chain(
            entries
                .iter()
                .filter(|_| cfg.mode != "direct")
                .map(|url| (url, true)),
        )
    {
        if !seen.insert(url) {
            continue;
        }
        let Some(session) = sessions.entries.get(url).filter(|s| s.expires_at > time()) else {
            continue;
        };
        let queue = match Queue::open(&api, &devices, url, session) {
            Ok(q) => q,
            Err(e) => {
                last_error = e;
                continue;
            }
        };
        // After submission the owner can already be executing the task. Poll
        // that same authority and retain the identifier on uncertain outcomes.
        return submit(&queue, &cfg, id, &body, queued);
    }
    Err(last_error)
}

fn submit(
    queue: &Queue<'_>,
    cfg: &Config,
    id: Uuid,
    body: &Value,
    queued: bool,
) -> Result<(), StateError> {
    let url = queue.url;
    let hint = format!(
        "owner={url} task_id={id} owner_id={} epoch={}",
        queue.authority.owner_id, queue.authority.epoch
    );
    let value = queue
        .request(Method::POST, TASKS, Some(body))
        .map_err(|e| format!("{e}; {hint} may_have_run=true"))?;
    let mut record = Record::read(value, id)?;
    let deadline = Duration::from_millis(cfg.total_timeout_ms);
    let stall = Duration::from_millis(cfg.stall_timeout_ms);
    let interval = Duration::from_millis(cfg.poll_interval_ms.clamp(10, 5000));
    let start = Instant::now();
    let mut changed = start;
    let mut maintained = start;
    while matches!(record.status.as_str(), "pending" | "leased") {
        if start.elapsed() >= deadline || changed.elapsed() >= stall {
            return Err(format!("task still pending; {hint} may_have_run=true").into());
        }
        std::thread::sleep(interval.min(deadline.saturating_sub(start.elapsed())));
        if queue.device.is_some() && maintained.elapsed() >= Duration::from_secs(10) {
            queue.devices.maintain();
            maintained = Instant::now();
        }
        match queue.record(id) {
            Ok(next) => {
                if next.status != record.status || next.attempts != record.attempts {
                    changed = Instant::now();
                }
                record = next;
            }
            Err(e) if garlic_client_logic::retry_allowed(Some(&e.to_string())) => continue,
            Err(e) => return Err(format!("{e}; {hint}").into()),
        }
    }
    let route = if queued {
        format!("infinity:{url}")
    } else {
        url.to_owned()
    };
    eprintln!(
        "owner_id={} epoch={} recovered=false",
        queue.authority.owner_id, queue.authority.epoch
    );
    record.print(&route)
}

pub fn run_job_cli(args: &[&str]) -> Option<Result<(), StateError>> {
    match args.first().copied() {
        Some("echo" | "uppercase" | "reverse" | "http" | "status") => Some(execute(args)),
        _ => None,
    }
}

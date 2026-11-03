//! Garlic task protocol; transport-independent and suitable for desktop/mobile use.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Priority { High, Normal, Low }
impl Default for Priority { fn default() -> Self { Self::Normal } }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Operation { Echo, Uppercase, Reverse, Http }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExchangeRequest {
    /// Logical service identifier, configured on each Storage node.
    pub service: String,
    pub method: String,
    /// Relative URI beginning with /. Never accepts an absolute URL.
    pub path: String,
    #[serde(default)] pub headers: BTreeMap<String, String>,
    #[serde(default)] pub body_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExchangeResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskRequest {
    pub operation: Operation,
    #[serde(default)] pub input: String,
    #[serde(default)] pub exchange: Option<ExchangeRequest>,
    #[serde(default)] pub priority: Priority,
    #[serde(default = "Uuid::new_v4")] pub task_id: Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus { Pending, Leased, Completed, Failed }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskView {
    pub task: TaskRequest,
    pub status: TaskStatus,
    pub result: Option<String>,
    #[serde(default)] pub exchange: Option<ExchangeResponse>,
    #[serde(default)] pub error: Option<String>,
    pub attempts: u32,
}
impl TaskView {
    pub fn pending(task: TaskRequest) -> Self {
        Self { task, status: TaskStatus::Pending, result: None, exchange: None, error: None, attempts: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRequest { pub worker_id: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimResponse { pub task: Option<TaskRequest>, pub lease_id: Option<Uuid> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteRequest { pub task_id: Uuid, pub worker_id: String, pub lease_id: Uuid, pub result: TaskView }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health { pub service: String, pub healthy: bool }

pub const MAX_BODY_BYTES: usize = 1024 * 1024;

pub fn execute_text(task: &TaskRequest) -> TaskView {
    let mut view = TaskView::pending(task.clone());
    view.attempts = 1;
    match task.operation {
        Operation::Echo => view.result = Some(task.input.clone()),
        Operation::Uppercase => view.result = Some(task.input.to_uppercase()),
        Operation::Reverse => view.result = Some(task.input.chars().rev().collect()),
        Operation::Http => {
            view.status = TaskStatus::Failed;
            view.error = Some("HTTP exchange requires a configured Storage worker".into());
            return view;
        }
    }
    view.status = TaskStatus::Completed;
    view
}

pub fn safe_relative_path(path: &str) -> bool {
    path.starts_with('/') && !path.starts_with("//") && !path.contains('#')
        && !path.contains('\\') && !path.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn text_execution() {
        let t = TaskRequest { operation: Operation::Reverse, input: "abc".into(), exchange: None, priority: Priority::Normal, task_id: Uuid::new_v4() };
        assert_eq!(execute_text(&t).result.as_deref(), Some("cba"));
    }
    #[test] fn relative_paths_only() {
        assert!(safe_relative_path("/search?q=rust"));
        assert!(!safe_relative_path("https://example.com"));
        assert!(!safe_relative_path("//evil.example"));
    }
}

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationTaskStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationTaskKind {
    Install,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationTaskLogLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTaskLogEntry {
    pub at: DateTime<Utc>,
    pub level: ApplicationTaskLogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationTask {
    pub id: String,
    pub kind: ApplicationTaskKind,
    pub status: ApplicationTaskStatus,

    pub template_id: String,
    pub requested_name: Option<String>,
    pub instance_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,

    pub summary: Option<String>,
    pub logs: Vec<ApplicationTaskLogEntry>,
}

impl ApplicationTask {
    pub fn is_finished(&self) -> bool {
        matches!(
            self.status,
            ApplicationTaskStatus::Succeeded | ApplicationTaskStatus::Failed
        )
    }
}

#[derive(Clone, Default)]
pub struct ApplicationTaskManager {
    inner: Arc<RwLock<HashMap<String, ApplicationTask>>>,
}

static GLOBAL_TASK_MANAGER: OnceLock<ApplicationTaskManager> = OnceLock::new();

impl ApplicationTaskManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn global() -> &'static ApplicationTaskManager {
        GLOBAL_TASK_MANAGER.get_or_init(Self::new)
    }

    pub async fn create_install_task(
        &self,
        template_id: impl Into<String>,
        requested_name: Option<String>,
    ) -> ApplicationTask {
        let now = Utc::now();
        let id = Uuid::now_v7().to_string();

        let mut task = ApplicationTask {
            id: id.clone(),
            kind: ApplicationTaskKind::Install,
            status: ApplicationTaskStatus::Pending,
            template_id: template_id.into(),
            requested_name,
            instance_id: None,
            created_at: now,
            started_at: None,
            finished_at: None,
            summary: Some("Task created".to_string()),
            logs: Vec::new(),
        };

        task.logs.push(ApplicationTaskLogEntry {
            at: now,
            level: ApplicationTaskLogLevel::Info,
            message: "Install task created".to_string(),
        });

        let mut guard = self.inner.write().await;
        guard.insert(id, task.clone());

        task
    }

    pub async fn mark_running(
        &self,
        task_id: &str,
        message: impl Into<String>,
    ) -> Option<ApplicationTask> {
        let mut guard = self.inner.write().await;
        let task = guard.get_mut(task_id)?;

        let now = Utc::now();
        task.status = ApplicationTaskStatus::Running;
        task.started_at = task.started_at.or(Some(now));

        let message = message.into();
        task.summary = Some(message.clone());
        task.logs.push(ApplicationTaskLogEntry {
            at: now,
            level: ApplicationTaskLogLevel::Info,
            message,
        });

        Some(task.clone())
    }

    pub async fn append_log(
        &self,
        task_id: &str,
        level: ApplicationTaskLogLevel,
        message: impl Into<String>,
    ) -> Option<ApplicationTask> {
        let mut guard = self.inner.write().await;
        let task = guard.get_mut(task_id)?;

        task.logs.push(ApplicationTaskLogEntry {
            at: Utc::now(),
            level,
            message: message.into(),
        });

        Some(task.clone())
    }

    pub async fn mark_succeeded(
        &self,
        task_id: &str,
        instance_id: Option<Uuid>,
        summary: impl Into<String>,
    ) -> Option<ApplicationTask> {
        let mut guard = self.inner.write().await;
        let task = guard.get_mut(task_id)?;

        let now = Utc::now();
        let summary = summary.into();

        task.status = ApplicationTaskStatus::Succeeded;
        task.finished_at = Some(now);
        task.instance_id = instance_id;
        task.summary = Some(summary.clone());
        task.logs.push(ApplicationTaskLogEntry {
            at: now,
            level: ApplicationTaskLogLevel::Info,
            message: summary,
        });

        Some(task.clone())
    }

    pub async fn mark_failed(
        &self,
        task_id: &str,
        summary: impl Into<String>,
    ) -> Option<ApplicationTask> {
        let mut guard = self.inner.write().await;
        let task = guard.get_mut(task_id)?;

        let now = Utc::now();
        let summary = summary.into();

        task.status = ApplicationTaskStatus::Failed;
        task.finished_at = Some(now);
        task.summary = Some(summary.clone());
        task.logs.push(ApplicationTaskLogEntry {
            at: now,
            level: ApplicationTaskLogLevel::Error,
            message: summary,
        });

        Some(task.clone())
    }

    pub async fn get(&self, task_id: &str) -> Option<ApplicationTask> {
        let guard = self.inner.read().await;
        guard.get(task_id).cloned()
    }

    pub async fn list(&self) -> Vec<ApplicationTask> {
        let guard = self.inner.read().await;
        let mut out = guard.values().cloned().collect::<Vec<_>>();
        out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        out
    }

    pub async fn prune_finished_older_than(&self, minutes: i64) -> usize {
        let mut guard = self.inner.write().await;
        let cutoff = Utc::now() - Duration::minutes(minutes.max(1));

        let before = guard.len();
        guard.retain(|_, task| {
            if !task.is_finished() {
                return true;
            }
            task.finished_at.map(|at| at >= cutoff).unwrap_or(false)
        });

        before.saturating_sub(guard.len())
    }
}

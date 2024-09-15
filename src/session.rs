use std::sync::Arc;
use chrono::{DateTime, Local, Utc};
use serde::Serialize;
use tokio::process::Child;
use tokio::sync::RwLock;
use crate::config::ConfigEntry;
use chrono::serde::ts_seconds;

pub enum SessionStatus {
    Starting = 1 << 0,
    Running = 1 << 1,
    Stopping = 1 << 2,
    Stopped = 1 << 3,
    Crashed = (1 << 4) | (1 << 3),
}

#[derive(Serialize)]
pub struct Session {
    pub config: ConfigEntry,

    #[serde(skip_serializing)]
    pub status: Arc<RwLock<SessionStatus>>,

    #[serde(with = "ts_seconds")]
    pub started: DateTime<Utc>,

    #[serde(skip_serializing)]
    pub process: Arc<RwLock<Child>>,
}
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Serialize};
use tokio::process::Child;
use tokio::sync::RwLock;
use crate::config::ConfigEntry;
use chrono::serde::ts_seconds;

#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum SessionStatus {
    Starting = 1 << 0,
    Running = 1 << 1,
    Stopping = 1 << 2,
    Stopped = 1 << 3,
    Crashed = (1 << 4) | (1 << 3),
}

pub struct Session {
    pub config: ConfigEntry,
    pub status: Arc<RwLock<SessionStatus>>,
    pub started: DateTime<Utc>,
    pub process: Arc<RwLock<Child>>,
}

#[derive(Serialize, Debug)]
pub struct SerializableSession {
    pub config: ConfigEntry,

    #[serde(with = "ts_seconds")]
    pub started: DateTime<Utc>,

    pub status: SessionStatus
}

impl Session {
    pub async fn to_serializable(&self) -> SerializableSession {
        let status = self.status.read().await;
        SerializableSession {
            config: self.config.clone(),
            started: self.started.clone(),
            status: status.clone()
        }
    }
}
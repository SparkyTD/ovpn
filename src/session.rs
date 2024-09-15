use std::sync::Arc;
use chrono::{DateTime, Local};
use tokio::process::Child;
use tokio::sync::RwLock;
use crate::config::Config;

pub enum SessionStatus {
    Starting = 1 << 0,
    Running = 1 << 1,
    Stopping = 1 << 2,
    Stopped = 1 << 3,
    Crashed = (1 << 4) | (1 << 3),
}

pub struct Session {
    config: Config,
    status: Arc<RwLock<SessionStatus>>,
    started: DateTime<Local>,
    process: Arc<RwLock<Child>>,
}
use std::future::Future;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use crate::config::{ConfigEntry, ConfigManager};
use anyhow::{Context, Result};
use chrono::{Utc};
use log::{error, info};
use nix::sys::signal;
use nix::unistd::Pid;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::timeout;
use crate::paths::OPENVPN_PATH;
use crate::session::{Session, SessionStatus};
use crate::state::AppState;

pub struct SessionManager {}

impl SessionManager {
    pub fn new() -> SessionManager { Self {} }

    pub async fn start(&self, config: Box<ConfigEntry>, app_state: Arc<AppState>) -> Result<()> {
        let config_path = ConfigManager::get_config_path(config.clone().as_ref());
        let mut command = Command::new(OPENVPN_PATH);
        command.arg("--config");
        command.arg(config_path);

        unsafe {
            command.pre_exec(|| {
                nix::unistd::setsid().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                Ok(())
            });
        }

        let child_process = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start the child process")?;

        let session = Session {
            status: Arc::new(RwLock::new(SessionStatus::Starting)),
            config: config.as_ref().clone(),
            started: Utc::now(),
            process: Arc::new(RwLock::new(child_process)),
        };

        let process_id = session.process.read().await.id().unwrap();

        info!("Child process has been started (PID: {})", process_id);

        *app_state.active_session.write().await = Some(session);

        // TODO Post 'Starting'

        Ok(())
    }

    pub async fn stop(&self, app_state: Arc<AppState>) -> Result<()> {

        let active_session_guard = app_state.active_session.read().await;
        let mut active_session = active_session_guard.as_ref().unwrap();
        {
            let mut child_process = active_session.process.write().await;
            let pid = child_process.id().context("Failed to get process id")?;
            let pid = Pid::from_raw(pid as i32);

            // TODO Post 'Stopping'

            signal::kill(Pid::from_raw(-pid.as_raw()), signal::Signal::SIGINT)
                .context("Failed to send SIGINT to process")?;
            info!("Sent SIGINT to process (PID: {})", pid);

            match timeout(Duration::from_secs(5), child_process.wait()).await {
                Ok(status) => info!("Process {} exited with status {:?}", pid, status),
                Err(_) => {
                    error!("Process didn't exit within timeout, forcefully killing");
                    child_process.kill().await.context("Failed to kill child process")?;
                }
            }
        }
        drop(active_session_guard);

        // TODO Post 'Stopped'

        *app_state.active_session.write().await = None;
        Ok(())
    }
}
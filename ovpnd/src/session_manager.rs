use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use crate::config::{ConfigEntry, ConfigManager};
use anyhow::{Context, Result};
use chrono::{Utc};
use log::{error, info};
use nix::sys::{signal, wait};
use nix::unistd::Pid;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::timeout;
use common::paths::OPENVPN_PATH;
use crate::session::{Session, SessionStatus};
use crate::session::SessionStatus::Stopping;
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
        SessionManager::update_active_session(app_state.clone(), SessionStatus::Starting).await
            .expect("Failed to change the status of the active session");

        self.start_monitoring_process(app_state.clone()).await?;

        Ok(())
    }

    pub async fn stop(&self, app_state: Arc<AppState>) -> Result<()> {
        let active_session_guard = app_state.active_session.read().await;
        let active_session = active_session_guard.as_ref().unwrap();
        {
            let mut child_process = active_session.process.write().await;
            let pid = child_process.id().context("Failed to get process id")?;
            let pid = Pid::from_raw(pid as i32);

            SessionManager::update_active_session(app_state.clone(), SessionStatus::Stopping).await
                .expect("Failed to change the status of the active session");

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

        SessionManager::update_active_session(app_state.clone(), SessionStatus::Stopped).await
            .expect("Failed to change the status of the active session");

        *app_state.active_session.write().await = None;

        Ok(())
    }

    async fn start_monitoring_process(&self, app_state: Arc<AppState>) -> Result<()> {
        let active_session_guard = app_state.active_session.read().await;
        let active_session = active_session_guard.as_ref().unwrap();
        let mut process = active_session.process.write().await;
        let pid = process.id().context("Failed to get process id")?;

        let stdout = process.stdout.take()
            .context("Failed to get stdout from child process")?;
        let mut reader = BufReader::new(stdout).lines();
        let app_state_clone = Arc::clone(&app_state);
        tokio::spawn(async move {
            while let Some(line) = reader.next_line().await.unwrap_or(None) {
                println!("OUT >> {}", line);

                if line.contains("Initialization Sequence Completed") {
                    SessionManager::update_active_session(app_state_clone.clone(), SessionStatus::Running).await
                        .expect("Failed to change the status of the active session");
                }
            }
        });

        let stderr = process.stderr.take()
            .context("Failed to get stderr from child process")?;
        let mut reader = BufReader::new(stderr).lines();
        tokio::spawn(async move {
            while let Some(line) = reader.next_line().await.unwrap_or(None) {
                println!("ERR >> {}", line);
            }
        });

        let app_state_clone = Arc::clone(&app_state);
        tokio::spawn(async move {
            let pid = Pid::from_raw(pid as i32);
            match wait::waitpid(pid, None) {
                Ok(_) => {
                    let app_state_clone = app_state_clone.clone();
                    let active_session_guard = app_state_clone.active_session.read().await;
                    let active_session = active_session_guard.as_ref().unwrap();
                    let status = active_session.status.read().await;
                    if status.clone() == Stopping {
                        println!("The process was stopped by the daemon, skipping exit handling");
                        return;
                    }
                    drop(status);
                    drop(active_session_guard);

                    SessionManager::update_active_session(app_state_clone.clone(), SessionStatus::Stopped).await
                        .expect("Failed to change the status of the active session");
                    *app_state_clone.active_session.write().await = None;
                    println!(">>> Process has exited")
                }
                Err(_) => error!("Failed to wait for process"),
            }
        });

        Ok(())
    }

    async fn update_active_session(app_state: Arc<AppState>, status: SessionStatus) -> Result<()> {
        let app_state_clone = Arc::clone(&app_state);
        let active_session = app_state_clone.active_session.read().await;
        *(*active_session).as_ref().unwrap().status.write().await = status;

        let app_state_clone = Arc::clone(&app_state);
        let active_session = (*active_session).as_ref().unwrap().to_serializable().await;
        let mut socket_manager = app_state_clone.socket_manager.lock().await;
        socket_manager.broadcast_status_change(&active_session).await
            .expect("Failed to broadcast status change");

        Ok(())
    }
}
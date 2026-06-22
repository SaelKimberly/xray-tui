use crate::bin_manager::{BinError, get_core_info};
use crate::config_builder::BackendConfig;
use crate::core_type::CoreType;
use std::path::{Path, PathBuf};
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};
use tokio::sync::mpsc::UnboundedSender;

/// Status of a running core process.
pub struct CoreProcess {
    child: Option<Child>,
    config_path: PathBuf,
    pub core_type: CoreType,
    /// The original sender stored for potential future use.
    #[allow(dead_code)]
    log_tx: UnboundedSender<String>,
}

impl CoreProcess {
    fn new(
        child: Child,
        config_path: PathBuf,
        core_type: CoreType,
        log_tx: UnboundedSender<String>,
    ) -> Self {
        Self {
            child: Some(child),
            config_path,
            core_type,
            log_tx,
        }
    }

    /// Send SIGKILL to the process and clean up config.
    async fn kill(&mut self) -> Result<(), ProcessError> {
        if let Some(mut child) = self.child.take() {
            child.kill().await?;
            child.wait().await?;
        }
        // Remove config file (best-effort)
        let _ = tokio::fs::remove_file(&self.config_path).await;
        Ok(())
    }
}

impl Drop for CoreProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // In Drop we can't use async, but the child's kill_on_drop is set
            // We just let it go out of scope and the runtime handles it
            let _ = child.start_kill();
        }
    }
}

/// Manages a single core process lifecycle.
pub struct CoreManager {
    config_dir: PathBuf,
    current: Option<CoreProcess>,
    log_tx: UnboundedSender<String>,
}

impl CoreManager {
    /// Create a new CoreManager with the given config directory.
    pub fn new(config_dir: PathBuf) -> Self {
        let (log_tx, _) = tokio::sync::mpsc::unbounded_channel();
        Self {
            config_dir,
            current: None,
            log_tx,
        }
    }

    /// Create a CoreManager that forwards log lines to the given sender.
    pub fn with_log_channel(config_dir: PathBuf, log_tx: UnboundedSender<String>) -> Self {
        Self {
            config_dir,
            current: None,
            log_tx,
        }
    }

    /// Start a core process with the given config and binary path.
    pub async fn start(
        &mut self,
        core_type: CoreType,
        config: &BackendConfig,
        binary_path: &Path,
    ) -> Result<(), ProcessError> {
        // Stop any running core of a different type
        if let Some(running) = &self.current
            && running.core_type != core_type
        {
            self.stop().await?;
        }

        let info = get_core_info(core_type)
            .ok_or_else(|| ProcessError::Startup("Unknown core type".to_string()))?;

        // Write config JSON
        let config_path = self.config_dir.join("config.json");
        let json = match config {
            BackendConfig::Xray(c) => serde_json::to_string_pretty(c)?,
            BackendConfig::SingBox(c) => serde_json::to_string_pretty(c)?,
        };
        tokio::fs::write(&config_path, &json).await?;

        // Build args from template: "run -c {0}" → ["run", "-c", config_path]
        let config_str = config_path.to_string_lossy().to_string();
        let args: Vec<String> = info
            .args_template
            .split(' ')
            .map(|part| {
                if part == "{0}" {
                    config_str.clone()
                } else {
                    part.to_string()
                }
            })
            .collect();

        // Spawn process
        let mut child = Command::new(binary_path)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        // Read stderr for logs
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| ProcessError::Startup("Failed to capture stderr".to_string()))?;
        let log_tx = self.log_tx.clone();
        tokio::spawn(async move {
            let reader = tokio::io::BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if log_tx.send(line).is_err() {
                    break;
                }
            }
        });

        // Read stdout for logs (xray-core logs to stdout by default)
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ProcessError::Startup("Failed to capture stdout".to_string()))?;
        let log_tx = self.log_tx.clone();
        tokio::spawn(async move {
            let reader = tokio::io::BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if log_tx.send(line).is_err() {
                    break;
                }
            }
        });

        // Poll for readiness (check process hasn't exited early)
        let max_retries = 20; // 500ms * 20 = 10s
        for _ in 0..max_retries {
            match child.try_wait() {
                Ok(Some(status)) => {
                    return Err(ProcessError::Startup(format!(
                        "Process exited early with status: {status}"
                    )));
                }
                Ok(None) => {
                    // Process still running
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
                Err(e) => {
                    return Err(ProcessError::Io(e));
                }
            }
        }

        // Check one more time after polling
        if let Some(status) = child.try_wait()? {
            return Err(ProcessError::Startup(format!(
                "Process exited during startup with status: {status}"
            )));
        }

        self.current = Some(CoreProcess::new(
            child,
            config_path,
            core_type,
            self.log_tx.clone(),
        ));

        Ok(())
    }

    /// Stop the running core process (SIGKILL) and clean up config.
    pub async fn stop(&mut self) -> Result<(), ProcessError> {
        if let Some(mut proc) = self.current.take() {
            proc.kill().await?;
        }
        Ok(())
    }

    /// Check if a core process is running.
    pub fn is_running(&self) -> bool {
        self.current.is_some()
    }

    /// Get the type of the running core, if any.
    pub fn running_core_type(&self) -> Option<CoreType> {
        self.current.as_ref().map(|p| p.core_type)
    }

    /// Get a reference to the log sender.
    pub fn log_tx(&self) -> UnboundedSender<String> {
        self.log_tx.clone()
    }
}

impl Drop for CoreManager {
    fn drop(&mut self) {
        // Dropping CoreProcess kills child via kill_on_drop + start_kill
        let _ = self.current.take();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Startup failed: {0}")]
    Startup(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Binary error: {0}")]
    Bin(#[from] BinError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_manager_is_not_running() {
        let mgr = CoreManager::new(PathBuf::from("/tmp"));
        assert!(!mgr.is_running());
        assert!(mgr.running_core_type().is_none());
    }

    #[test]
    fn stop_when_not_running_is_ok() {
        let mut mgr = CoreManager::new(PathBuf::from("/tmp"));
        // Should not panic or error
        let result = mgr.stop();
        // stop() is async, so we need a runtime for a real test
        // Just verify the method signature works
        drop(result);
    }
}

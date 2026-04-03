use std::path::PathBuf;
use tokio::process::{Child, Command};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json;
use reqwest;
use log::{info, error};

use crate::errors::RcloneError;


/// Spawns the rclone rcd daemon with the given configuration and returns a handle to it.
#[derive(Debug)]
pub struct RcloneDaemon {

    // info for spawning the rclone rcd process
    rclone_path: PathBuf,
    rc_addr: String, // "127.0.0.1:5572"

    // living handle to the child process so we can kill it later
    inner: Arc<Mutex<Option<Child>>>,

    // seconds to wait for start / stop of deamon
    delta: u64,
}


impl RcloneDaemon {
    pub fn new(rclone_path: impl Into<PathBuf>) -> Self {
        Self {
            rclone_path: rclone_path.into(),
            rc_addr: "127.0.0.1:5572".into(),
            inner: Arc::new(Mutex::new(None)),
            delta: 5u64,
        }
    }

    // spawn rclone rcd in the background
    pub async fn start(&self) -> Result<(), RcloneError> {
        
        let mut guard = self.inner.lock().await;

        let child = Command::new(&self.rclone_path)
            .args([
                "rcd",
                "--rc-addr", &self.rc_addr,
                "--rc-no-auth",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| RcloneError::DaemonFailedToStart(e.to_string()))?;
        
        // spawned child process stored in mutex
        *guard = Some(child);

        // immediatly release
        drop(guard);

        // poll until rclone RC is actually accepting connections
        let client = reqwest::Client::new();

        // noop: no-operation, just used to check if the RC is responsive
        let url = format!("http://{}/rc/noop", self.rc_addr);
        
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(self.delta);
        loop {
            if tokio::time::Instant::now() > deadline {
                // clean up the process since it never became ready
                self.stop().await?;
                return Err(RcloneError::Timeout);
            }

            match client.post(&url).send().await {
                Ok(resp) if resp.status().is_success() => break,
                _ => tokio::time::sleep(std::time::Duration::from_millis(50)).await,
            }
        }

        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        let client = reqwest::Client::new();
        let url = format!("http://{}/rc/noop", self.rc_addr);
        client.post(&url).send().await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    // kill the rcd process when shutting down
    pub async fn stop(&self) -> Result<(), RcloneError> {
        let mut guard = self.inner.lock().await;

        if let Some(mut child) = guard.take() {
            // kill errors are safe to ignore for an owned child process:
            // ESRCH = already dead, EPERM = impossible for own child.
            let _ = child.kill().await;
            // always wait to reap the process entry from the OS.
            let _ = child.wait().await;
        }

        Ok(())
    }

    /// query the rclone HTTP API (generic helper)
    /// any ok/error must be logged so every call is traceable
    /// 
    /// TODO: create more error distinction based on error type (network, parse, daemon)
    /// TODO: add timeout to prevent hanging calls
    pub async fn call_rc<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T, RcloneError> {

        // log the call with params, so we have a full trace of all executed RC calls
        info!("[RC] → {} params={}", method, params);

        let client = reqwest::Client::new();
        let url = format!("http://{}/{}", self.rc_addr, method);

        // send the request to the daemon and wait for response
        let resp = client
            .post(&url)
            .json(&params)
            .send()
            .await
            .map_err(|e| RcloneError::RcCallFailed(e.to_string()))?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| RcloneError::RcCallFailed(e.to_string()))?;

        // daemon replies with error status code
        if !status.is_success() {
            error!("[RC] : {} error=http_{} response={}", method, status.as_u16(), text);
            return Err(RcloneError::RcCallFailed(format!("http {}: {}", status.as_u16(), text)));
        }

        // try to parse the response body as JSON
        let result = serde_json::from_str::<T>(&text)
            .map_err(|e| {
                error!("[RC] : {} error=parse_failed response={}", method, text);
                RcloneError::RcCallFailed(e.to_string() + &format!(" response={}", text))
            })?;

        info!("[RC] : {} ok", method);
        Ok(result)
    }
}

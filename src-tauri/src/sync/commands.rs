use std::collections::HashMap;
use std::sync::Arc;

use tauri::Manager;
use crate::sync::daemon::RcloneDaemon;
use crate::sync::commands_inner::{get_remotes_inner, rclone_check};
use crate::tray::manager::TrayManager;
use crate::errors::BackendError;
use crate::types::RemoteInfo;


/// Pings the rclone RC daemon via `rc/noop` to check if it is responsive.
/// Returns `true` if the daemon replies successfully, `false` otherwise.
#[tauri::command]
pub async fn daemon_running(daemon: tauri::State<'_, Arc<RcloneDaemon>>) -> Result<bool, BackendError> {
    Ok(daemon.is_running().await)
}

/// Returns the rclone version string for display in the navbar,
/// or an error if rclone is not available.
#[tauri::command]
pub async fn get_rclone_version() -> Result<String, BackendError> {
    Ok(rclone_check().await?)
}

/// Returns all configured rclone remotes with their type to the frontend.
#[tauri::command]
pub async fn get_remotes(daemon: tauri::State<'_, Arc<RcloneDaemon>>) -> Result<Vec<RemoteInfo>, BackendError> {
    Ok(get_remotes_inner(&daemon).await?)
}

/// Stops the rclone RC daemon and clears the tray.
#[tauri::command]
pub async fn stop_daemon(app: tauri::AppHandle, daemon: tauri::State<'_, Arc<RcloneDaemon>>) -> Result<(), BackendError> {
    daemon.stop().await?;
    app.state::<TrayManager>().set_remotes(&app, HashMap::new());
    Ok(())
}

/// Restarts the rclone RC daemon, clears the tray, and repopulates it.
#[tauri::command]
pub async fn restart_daemon(app: tauri::AppHandle, daemon: tauri::State<'_, Arc<RcloneDaemon>>) -> Result<(), BackendError> {
    daemon.stop().await?;
    app.state::<TrayManager>().set_remotes(&app, HashMap::new());
    daemon.start().await?;
    let remotes = get_remotes_inner(&daemon).await?;
    let statuses = crate::store::init::load_statuses(&app, &remotes);
    app.state::<TrayManager>().set_remotes(&app, statuses);
    Ok(())
}

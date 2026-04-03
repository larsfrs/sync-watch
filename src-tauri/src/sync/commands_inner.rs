use crate::sync::daemon::RcloneDaemon;
use crate::errors::RcloneError;
use crate::types::{RemoteInfo, RemoteEntry};


/// Little helper to unpack strings from a serde_json::Value response, with error handling.
fn get_string(resp: &serde_json::Value, field: &str) -> Result<String, RcloneError> {
    resp.get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| RcloneError::UnexpectedResponse(
            format!("missing or invalid '{}' field in response: {}", field, resp)
        ))
}


/// Helper to unpack arrays of strings from a serde_json::Value response.
fn get_trimmed_array(resp: &serde_json::Value, field: &str) -> Result<Vec<String>, RcloneError> {
    resp.get(field)
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.trim_end_matches(':').to_string())
            .collect())
        .ok_or_else(|| RcloneError::UnexpectedResponse(
            format!("missing or invalid '{}' field in response: {}", field, resp)
        ))
}

/// Returns all configured rclone remotes with their type.
///
/// Makes two RC calls per remote:
///
/// **1. `config/listremotes` — get all remote names**
/// ```json
/// // response
/// { "remotes": ["gdrive:", "nas:"] }
/// ```
/// The trailing `:` is stripped, giving `["gdrive", "nas"]`.
///
/// **2. `config/get` per remote — get its type**
/// ```json
/// // request
/// { "name": "gdrive" }
///
/// // response (full rclone config entry, we only use "type")
/// { "type": "drive", "client_id": "...", "scope": "drive", "token": "..." }
/// ```
///
/// **Rust return value:** `Vec<RemoteInfo>` where each entry is `{ name, type }`.
/// Tauri automatically serializes this to JSON via serde before passing it to the frontend,
/// so the frontend receives:
/// ```json
/// [
///   { "name": "gdrive", "type": "drive" },
///   { "name": "nas",    "type": "sftp"  }
/// ]
/// ```
pub async fn get_remotes_inner(daemon: &RcloneDaemon) -> Result<Vec<RemoteInfo>, RcloneError> {
    
    // first call: get remote names
    let response = daemon.call_rc::<serde_json::Value>(
        "config/listremotes", serde_json::json!({})
    ).await?;

    // unpack and trim trailing ':' from remote names (e.g. "gdrive:" -> "gdrive")
    let names: Vec<String> = get_trimmed_array(&response, "remotes")?;

    // second call per remote: get its type
    let mut remotes = Vec::new();
    for name in names {
        
        let config = daemon
            .call_rc::<serde_json::Value>("config/get", serde_json::json!({ "name": name }))
            .await
            .map_err(|e| RcloneError::RcCallFailed(format!("Failed to get config for {name}: {e:?}")))?;
        
        remotes.push(RemoteInfo {
            r#type: get_string(&config, "type")?,
            name,
        });
    }
    Ok(remotes)
}


/// Builds the RC params for a bisync call.
/// Pass `resync: true` on the first ever run for a remote to initialise bisync state files.
pub fn bisync_rc_params(remote: &str, config: &RemoteEntry, resync: bool) -> serde_json::Value {
    let mut params = serde_json::json!({
        "path1": config.local_path,
        "path2": format!("{}:{}", remote, config.remote_path),
        "_group": format!("bisync:{}", remote),
        "_config": {
            "ConflictResolve": config.conflict_resolve,
            "ConflictLoser":   config.conflict_loser,
        }
    });
    if resync {
        params["resync"] = serde_json::json!(true);
    }
    params
}

/// Checks if rclone is installed by running `rclone version` as a subprocess.
/// Returns the first line of its output (e.g. `"rclone v1.68.2"`) on success,
/// or a [`RcloneError`] if the binary is not found or exits with an error.
///
/// ### Example output
/// ```text
/// Ok("rclone v1.68.2")
/// Err(Other("Rclone is not installed or not found in PATH"))
/// ```
pub async fn rclone_check() -> Result<String, RcloneError> {
    std::process::Command::new("rclone")
        .arg("version")
        .output()
        .map_err(|e| RcloneError::InstalledCheckFail(format!("Failed to check rclone version: {e:?}")))
        .and_then(|output| {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("rclone")
                    .to_string())
            } else {
                Err(RcloneError::NotInstalled)
            }
        })
}
use crate::errors::RcloneError;


/// The current sync state of a remote.
/// Serialized with an adjacently tagged representation so the frontend
/// can switch on `kind` and read `data` cleanly.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum SyncStatus {
    UpToDate,
    Syncing,
    Transferred { transferred: u64 },
    AutoResolved { transferred: u64, conflicts: u64 },
    ActionRequired,
    Error { message: String },
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::UpToDate => write!(f, "Up to date."),
            SyncStatus::Syncing => write!(f, "Syncing..."),
            SyncStatus::Transferred { transferred } => write!(f, "↑↓ {}", transferred),
            SyncStatus::AutoResolved { transferred, conflicts } => write!(f, "↑↓ {} : {}↯ auto-resolved", transferred, conflicts),
            SyncStatus::ActionRequired => write!(f, "↯ Action required"),
            SyncStatus::Error { message } => write!(f, "Error: {}", message),
        }
    }
}

/// A little unpack helper.
fn get_u64(stats: &serde_json::Value, field: &str) -> Result<u64, RcloneError> {
    stats.get(field)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| RcloneError::UnexpectedResponse(
            format!("missing or invalid '{}' field in stats: {}", field, stats)
        ))
}

/// Derives the tray status string from a completed bisync response.
/// Later, this will show up in the tray menu. 
pub fn create_sync_status(
    stats: &serde_json::Value,
    conflict_resolve: &str
) -> Result<SyncStatus, RcloneError> {

    let transfers = get_u64(stats, "transfers")?;
    let errors = get_u64(stats, "errors")?;

    if errors == 0 && transfers == 0 {
        return Ok(SyncStatus::UpToDate);
    }

    if transfers > 0 && errors == 0 {
        return Ok(SyncStatus::Transferred { transferred: transfers });
    }

    if transfers > 0 && errors > 0 && conflict_resolve == "none" {
        return Ok(SyncStatus::ActionRequired);
    }

    if transfers > 0 && errors > 0 && conflict_resolve != "none" {
        return Ok(SyncStatus::AutoResolved { transferred: transfers, conflicts: errors });
    }

    Err(RcloneError::UnexpectedResponse(
        format!("unexpected stats response from rc daemon: {}", stats)
    ))
}

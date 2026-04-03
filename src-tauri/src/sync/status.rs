use crate::errors::RcloneError;
use crate::types::SyncStatus;


/// A little unpack helper.
fn get_u64(stats: &serde_json::Value, field: &str) -> Result<u64, RcloneError> {
    stats.get(field)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| RcloneError::UnexpectedResponse(
            format!("missing or invalid '{}' field in stats: {}", field, stats)
        ))
}

/// Derives the sync status from a completed bisync response.
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

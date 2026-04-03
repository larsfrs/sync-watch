#![allow(unused_doc_comments)]

/// ### Structure:
/// We have one main type, [`RemoteEntry`], which represents all the data we want to track
/// for each rclone remote. Then, we have a status enum [`SyncStatus`] which
/// is a field in RemoteEntry but also used separately in the tray and frontend
/// to represent the current sync state of a remote.
/// 
/// [`RemoteInfo`] is a simpler struct that only contains the remote name
/// and type as reported by rclone. Not persisted, filled when we read the
/// remotes from rclone on startup.


/// All persisted state for a remote: user config + runtime state.
/// One entry per remote in the store under the "remotes" key.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RemoteEntry {
    // user config
    pub locked: bool,
    pub local_path: String,
    pub remote_path: String,
    pub timer_enabled: bool,
    pub poll_interval: u32,
    pub watch_enabled: bool,
    pub watch_debounce: u32,
    pub conflict_resolve: String,
    pub conflict_loser: String,
    // runtime state
    pub status: SyncStatus,
    pub bisync_initialized: bool,
}

impl Default for RemoteEntry {
    fn default() -> Self {
        Self {
            locked: false,
            local_path: String::new(),
            remote_path: String::new(),
            timer_enabled: false,
            poll_interval: 60,
            watch_enabled: false,
            watch_debounce: 5,
            conflict_resolve: "none".to_string(),
            conflict_loser: "num".to_string(),
            status: SyncStatus::UpToDate,
            bisync_initialized: false,
        }
    }
}

/// The current sync state of a remote.
/// Serialized with an adjacently tagged representation so the frontend
/// can switch on `kind` and read `data` cleanly.
/// Defaults to `UpToDate` if not yet set.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum SyncStatus {
    #[default]
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


/// Information about remotes that we get from rclone,
/// and only use at runtime, not persisted.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RemoteInfo {
    pub name: String,
    pub r#type: String,
}
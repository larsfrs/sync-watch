/// Information about remotes that the backend and frontend share.
/// Needs to be handled centrally over the backend, so theres no diverging versions
/// in front and backend. Also, 
/// 
/// Change to camelCase for idiomatic TS when serializing with serde.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RemoteConfig {
    pub locked: bool,
    pub local_path: String,
    pub remote_path: String,
    pub timer_enabled: bool,
    pub poll_interval: u32,
    pub watch_enabled: bool,
    pub watch_debounce: u32,
    pub conflict_resolve: String,
    pub conflict_loser: String,
}

impl Default for RemoteConfig {
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
        }
    }
}
use std::sync::Arc;
use tauri::{App, Manager};

use crate::sync::daemon::RcloneDaemon;


/// Creates the rclone daemon and registers it with app state.
/// Does not start it, startup is orchestrated by lib.rs.
pub fn init(app: &mut App) {
    let daemon = Arc::new(RcloneDaemon::new("rclone"));
    app.manage(daemon);
}

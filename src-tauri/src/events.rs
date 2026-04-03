use tauri::{Manager, AppHandle};
use std::sync::Arc;

use crate::sync::daemon::RcloneDaemon;

pub fn handle(app: &AppHandle, event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::Exit => {
            let daemon = app.state::<Arc<RcloneDaemon>>();
            tauri::async_runtime::block_on(async {
                let _ = daemon.stop().await;
            });
        }

        // add other cases here
        _ => {}
    }
}
#![allow(unused_doc_comments)]

mod sync;
mod store;
mod tray;

// single file modules
mod errors;
mod events;
mod types;

use std::sync::Arc;
use sync::commands::{daemon_running, get_rclone_version, get_remotes, stop_daemon, restart_daemon};
use sync::daemon::RcloneDaemon;
use sync::commands_inner::get_remotes_inner;
use tray::manager::TrayManager;
use errors::BackendError;

use tauri::{Manager, Emitter};
use tauri::async_runtime::spawn;
use tauri_plugin_store::StoreExt;
use tauri_plugin_log::{Target, TargetKind};


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()

        // setup log, store, opener, and dialog plugins for tauri
        .plugin(tauri_plugin_log::Builder::new()
            .targets([Target::new(TargetKind::LogDir {
                file_name: Some("sync-watch".to_string())
            })])

            // Limit log file size to 10mb. "KeepOne" retains one backup before rotating.
            // TODO: make this configurable in the UI.
            .max_file_size(10_000_000)
            .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
            .build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())

        .setup(|app| {

            /// The setup follows a phased approach:
            /// 1. Initialize infrastructure: store, tray, daemon,
            ///    sync engine (sync engine will hold file watcher and timers for later)
            /// 2. Start the daemon in the background
            /// 3. Once the daemon is running, load data (config data, remotes from rclone) and fill the tray
            /// (4.1) TODO: Start the sync engine.
            /// (4.2) TODO: start file watcher and timers if active for any remotes, this has to be done
            ///            after starting deamon & loading data.

            // phase 1: infrastructure
            let _ = app.store("app_data.json"); // initialize store plugin with a file
            TrayManager::build(app)?; // initialize tray manager and register with app state
            
            // init sync related infrastructure and register with app state:
            // - rclone deamon, sync engine
            sync::init::init(app);

            // phase 2+3: async background: daemon start, then tray population
            let app_handle = app.handle().clone();
            spawn(async move {
                let daemon = app_handle.state::<Arc<RcloneDaemon>>();

                // phase 2: start daemon
                if let Err(e) = daemon.start().await {
                    let _ = app_handle.emit("error", BackendError::from(e));
                    return;
                }

                // phase 3: fetch remotes, load persisted state, fill tray
                match get_remotes_inner(&daemon).await {
                    Err(e) => { let _ = app_handle.emit("error", BackendError::from(e)); }
                    Ok(remotes) => {
                        let statuses = store::init::load_statuses(&app_handle, &remotes);
                        app_handle.state::<TrayManager>().set_remotes(&app_handle, statuses);
                    }
                }

                // TODO: start file watcher if active for any remotes
                // TODO: start timers if active for any remotes
            });

            Ok(())
        })

        // all backend commands exposed to the frontend:
        .invoke_handler(tauri::generate_handler![

            // sync related commands
            daemon_running, stop_daemon, restart_daemon, get_rclone_version, get_remotes

            // TODO: remote config commands
            // get_remote_config, set_remote_config, get_remote_status, get_remote_history

        ])

        .build(tauri::generate_context!())
        .expect("error while running tauri application")

        .run(|app, event| events::handle(app, event));
}

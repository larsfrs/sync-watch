use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
use tauri::{App, AppHandle, Manager, image::Image};
use tauri::menu::{Menu, MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use crate::types::{RemoteEntry, SyncStatus};
use crate::store;


pub struct TrayManager {
    tray: tauri::tray::TrayIcon,
    
    // BTreeMap keeps remotes sorted alphabetically in the tray.
    statuses: Mutex<BTreeMap<String, SyncStatus>>,
}

/// Usage of the Tray Manager:
/// 
/// 1. The Tray manager gets constructed in lib.rs via `<tray::manager::TrayManager::build(app)?;`
/// 2. The Tray Manager gets filled with data via the app handle: `app_handle.state::<TrayManager>().set_remotes(&app_handle, statuses);`
impl TrayManager {

    /// Creates the tray icon, wires the menu event handler, and registers with app state.
    pub fn build(app: &mut App) -> tauri::Result<()> {
        let tray = TrayIconBuilder::new()
            .icon(Image::from_bytes(include_bytes!("../../../assets/32x32_icon.png"))?)
            .tooltip("sync-watch")
            .on_menu_event(Self::handle_menu_event)
            .build(app)?;

        app.manage(TrayManager {
            tray,
            statuses: Mutex::new(BTreeMap::new()),
        });

        Ok(())
    }

    /// Replaces the tracked remote list with the provided one, then rebuilds the tray menu.
    pub fn set_remotes(&self, app: &AppHandle, statuses_in: HashMap<String, SyncStatus>) {
        let mut statuses = self.statuses.lock().unwrap();
        *statuses = statuses_in.into_iter().collect();
        drop(statuses);
        self.rebuild_menu(app);
    }

    /// Updates a single remote's status and rebuilds the tray menu.
    pub fn set_status(&self, app: &AppHandle, remote: &str, status: SyncStatus) {
        let mut statuses = self.statuses.lock().unwrap();
        if statuses.contains_key(remote) {
            statuses.insert(remote.to_string(), status);
            drop(statuses);
            self.rebuild_menu(app);
        }
    }

    fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
        let id = event.id().as_ref();
        if let Some(remote) = id.strip_prefix("remote::") {
            Self::handle_remote_click(app, remote);
        } else {
            match id {
                "open_app" => Self::show_main_window(app),
                "quit" => app.exit(0),
                _ => {}
            }
        }
    }

    fn handle_remote_click(app: &AppHandle, remote: &str) {
        let configs: HashMap<String, RemoteEntry> =
            store::load(app, "remotes").unwrap_or_default();
        if configs.get(remote).map(|c| c.locked).unwrap_or(false) { return; }
        let app = app.clone();
        let remote = remote.to_string();
        tauri::async_runtime::spawn(async move {
            // TODO: trigger sync for remote
            let _ = remote;
            let _ = app;
        });
    }

    fn show_main_window(app: &AppHandle) {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.unminimize();
            let _ = w.show();
            let _ = w.set_focus();
        }
    }

    fn rebuild_menu(&self, app: &AppHandle) {
        let statuses = self.statuses.lock().unwrap();
        let configs: HashMap<String, RemoteEntry> =
            store::load(app, "remotes").unwrap_or_default();
        if let Ok(menu) = self.build_menu(app, &statuses, &configs) {
            let _ = self.tray.set_menu(Some(menu));
        }
    }

    fn build_menu(
        &self,
        app: &AppHandle,
        statuses: &BTreeMap<String, SyncStatus>,
        configs: &HashMap<String, RemoteEntry>,
    ) -> tauri::Result<Menu<tauri::Wry>> {
        let mut builder = MenuBuilder::new(app);

        for (remote, status) in statuses.iter() {
            let is_locked = configs.get(remote).map(|c| c.locked).unwrap_or(false);
            let label = if is_locked {
                format!("[L] {}", remote)
            } else {
                format!("{}:  {}", remote, status)
            };
            let item = MenuItemBuilder::with_id(format!("remote::{}", remote), label).build(app)?;
            if is_locked { item.set_enabled(false)?; }
            builder = builder.item(&item);
        }

        builder
            .separator()
            .item(&MenuItemBuilder::with_id("open_app", "Open App").build(app)?)
            .item(&MenuItemBuilder::with_id("quit", "Quit").build(app)?)
            .build()
    }
}

# <img src="assets/icon.svg" width="32" style="vertical-align: middle"> sync-watch

- Tauri Wrapper for Rclone that handles file watching and remote polling.
- Manages Rclone remotes like a native cloud sync-client.

## Features 

- This action diagram encapsulates the whole logic of sync-watch, using "rem1" as
an example remote:

```mermaid
flowchart TD
    %% Triggers
    FW[File watcher\ndetects local change]
    PT[Poll timer fires]
    MB[User clicks remote\nstatus in tray]

    FW --> DB[Debounce 2s (customizable)]
    DB --> GS
    PT --> GS
    MB --> GS

    %% Guard
    GS{is_syncing?}
    GS -->|yes| SKIP[Skip trigger]
    GS -->|no| LOCK[Lock: is_syncing = true\nTray item: 'Syncing...']

    %% Execute
    LOCK --> RUN["sync/bisync via RC
    --conflict-resolve {cfg}
    --conflict-loser {cfg}"]

    %% Result branches
    RUN --> R{Result}

    R -->|success, nothing changed| A["Tray: 'rem1: Up to date.'"]
    R -->|files transferred, no conflicts| B["Tray: 'rem1: ↑↓n'"]
    R -->|conflicts auto-resolved| C["Tray: 'rem1: ↑↓n : n↯ auto-resolved'
    OS notification: N conflicts auto-resolved for rem1"]
    R -->|conflicts, resolve=none| D["Tray: 'rem1: ↯ Action required'
    OS notification: N conflicts need manual resolution in rem1"]
    R -->|error| E["Tray: 'Error'
    OS notification: sync error on rem1"]

    A & B & C & D & E --> UNLOCK[Unlock: is_syncing = false]
```


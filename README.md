<h1>
  <img src="assets/icon.svg" alt="Icon" height="36" align="absmiddle"> 
  sync-watch
</h1>

<img align="right" src="assets/demo.gif" width="25%" alt="Demo Animation">

- Rclone Wrapper written in Rust; Handles file watching and remote polling.<br>
- Manages Rclone remotes like a native cloud sync-client.<br>
- Provides a system tray interface for status and manual sync triggers.

## Features 

- This activity diagram encapsulates the whole logic of sync-watch, using "rem1" as
an example remote:

```mermaid
flowchart TD
    %% Triggers
    FW[File watcher\ndetects local change]
    PT[Poll timer fires]
    MB[User clicks remote\nstatus in tray]

    FW --> DB["Debounce 2s (customizable)"]
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
## Dependencies

## Installation
- Prerequisite: [Rclone](https://rclone.org/downloads/) must be installed.

## Development Setup

## Limitations

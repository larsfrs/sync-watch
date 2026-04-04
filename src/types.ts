export interface RemoteInfo {
    name: string;
    type: string;
}

export interface RemoteConfig {
    locked: boolean;
    localPath: string;
    remotePath: string;
    timerEnabled: boolean;
    pollInterval: number;
    watchEnabled: boolean;
    watchDebounce: number;
    conflictResolve: string;
    conflictLoser: string;
}

export type SyncStatus =
    | { kind: "upToDate" }
    | { kind: "syncing" }
    | { kind: "transferred";
        data: { transferred: number }
      }
    | { kind: "autoResolved";
        data: { transferred: number; conflicts: number }
      }
    | { kind: "actionRequired" }
    | { kind: "error";
        data: { message: string }
      };

export type BackendError = { kind: string; message: unknown };

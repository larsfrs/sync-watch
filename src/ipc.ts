import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { RemoteInfo, BackendError } from "./types";

export async function getRemotes(): Promise<RemoteInfo[]> {
    return invoke("get_remotes");
}

export async function daemonRunning(): Promise<boolean> {
    return invoke("daemon_running");
}

export async function getRcloneVersion(): Promise<string> {
    return invoke("get_rclone_version");
}

export function hideWindow(): Promise<void> {
    return getCurrentWindow().hide();
}

export async function stopDaemon(): Promise<void> {
    return invoke("stop_daemon");
}

export async function restartDaemon(): Promise<void> {
    return invoke("restart_daemon");
}

export function onError(cb: (e: BackendError) => void): void {
    listen<BackendError>("error", ({ payload }) => cb(payload));
}
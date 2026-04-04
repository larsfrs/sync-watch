import { getRemotes, getRcloneVersion, daemonRunning, hideWindow, stopDaemon, restartDaemon, onError } from "./ipc";
import { renderRemote } from "./remote";
import { showError } from "./error";


window.addEventListener("DOMContentLoaded", async () => {
    const dot  = document.getElementById("daemon-status")!;
    const list = document.getElementById("remote-list")!;

    onError(showError);

    document.getElementById("hide-btn")!.addEventListener("click", hideWindow);
    document.getElementById("stop-btn")!.addEventListener("click", () =>
        stopDaemon().then(updateDot).catch(showError)
    );
    document.getElementById("restart-btn")!.addEventListener("click", () =>
        restartDaemon().then(updateDot).catch(showError)
    );

    const updateDot = async () => {
        // animate "." -> ".." -> "..." while querying
        dot.textContent = ".";
        const dotInterval = setInterval(() => {
            const len = (dot.textContent?.length ?? 0) % 3 + 1;
            dot.textContent = ".".repeat(len);
        }, 400);

        // run query and minimum display time in parallel
        const [online] = await Promise.all([
            daemonRunning().catch(() => false),
            new Promise(r => setTimeout(r, 800)),
        ]);

        clearInterval(dotInterval);
        dot.textContent = "Query daemon";
        dot.classList.toggle("online",  online as boolean);
        dot.classList.toggle("offline", !(online as boolean));
    };

    await updateDot();
    dot.addEventListener("click", updateDot);

    // toggle collapsible daemon controls
    const pillToggle = document.getElementById("pill-toggle")!;
    const pillControls = document.getElementById("pill-controls")!;
    pillToggle.addEventListener("click", () => {
        pillControls.classList.toggle("expanded");
    });

    const appName = document.getElementById("app-name")!;
    getRcloneVersion()
        .then(v => appName.textContent = v)
        .catch(() => appName.textContent = "rclone not found");

    const loadRemotes = async () => {
        list.innerHTML = "";
        const remotes = await getRemotes().catch(e => { showError(e); return []; });
        for (const remote of remotes) {
            list.appendChild(renderRemote(remote));
        }
    };

    document.getElementById("refresh-btn")!.addEventListener("click", loadRemotes);
    await loadRemotes();
});

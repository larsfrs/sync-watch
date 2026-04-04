import { BackendError } from "./types";

export function showError(e: BackendError): void {
    const toast = document.createElement("div");
    toast.className = "toast";
    toast.innerHTML = `
        <span>${e.kind}: ${JSON.stringify(e.message)}</span>
        <button class="toast-close">x</button>
    `;
    toast.querySelector(".toast-close")!.addEventListener("click", () => toast.remove());
    document.getElementById("toasts")!.appendChild(toast);
}

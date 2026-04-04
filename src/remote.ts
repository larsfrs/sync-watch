import { RemoteInfo } from "./types";

export function renderRemote(info: RemoteInfo): HTMLElement {
    const card = document.createElement("div");
    card.className = "remote-card";
    card.innerHTML = `
        <span class="remote-name">${info.name}</span>
        <span class="remote-type">${info.type}</span>
    `;
    return card;
}

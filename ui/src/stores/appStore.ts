import { writable } from "svelte/store";

export const projectId = writable<string | null>(null);
export const activeTab = writable<string>("projects");
export const mainView = writable<"graph" | "ctx">("graph");
export const filterPanelOpen = writable<boolean>(true);

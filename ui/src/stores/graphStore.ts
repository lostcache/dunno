import { writable } from "svelte/store";
import type cytoscape from "cytoscape";
import type { NodeData } from "../lib/types";

export const cyInstance = writable<cytoscape.Core | null>(null);
export const editingNode = writable<NodeData | null>(null);
export const hoverNode = writable<NodeData | null>(null);

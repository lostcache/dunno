import { writable } from "svelte/store";
import type { NodeData } from "../lib/types";

export type ModalMode = "create" | "edit" | null;

interface ModalState {
  open: boolean;
  mode: ModalMode;
  tab?: string;
  editingNode?: NodeData;
}

export const modalState = writable<ModalState>({ open: false, mode: null });

export function openCreate(tab: string): void {
  modalState.set({ open: true, mode: "create", tab });
}

export function openEdit(node: NodeData): void {
  modalState.set({ open: true, mode: "edit", editingNode: node });
}

export function closeModal(): void {
  modalState.set({ open: false, mode: null });
}

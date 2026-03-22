import { writable } from 'svelte/store'
import type { NodeData } from '../lib/types'

export type ModalMode = 'create' | 'edit' | 'add-link' | null

interface ModalState {
  open: boolean
  mode: ModalMode
  tab?: string
  editingNode?: NodeData
}

export const modalState = writable<ModalState>({ open: false, mode: null })

export function openCreate(tab: string): void {
  modalState.set({ open: true, mode: 'create', tab })
}

export function openEdit(node: NodeData): void {
  modalState.set({ open: true, mode: 'edit', editingNode: node })
}

export function openAddLink(node: NodeData): void {
  modalState.set({ open: true, mode: 'add-link', editingNode: node })
}

export function closeModal(): void {
  modalState.set({ open: false, mode: null })
}

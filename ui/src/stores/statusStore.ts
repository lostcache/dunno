import { writable } from 'svelte/store'

interface StatusState {
  message: string
  cls: '' | 'ok' | 'err'
}

export const statusState = writable<StatusState>({ message: 'Ready', cls: '' })

export function setStatus(msg: string, cls: '' | 'ok' | 'err' = ''): void {
  statusState.set({ message: msg, cls })
  if (cls === 'ok') {
    setTimeout(() => {
      statusState.update(s => s.message === msg ? { message: 'Ready', cls: '' } : s)
    }, 3000)
  }
}

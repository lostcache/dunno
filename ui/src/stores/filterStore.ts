import { writable } from 'svelte/store'

export const hiddenNodeTypes = writable<Set<string>>(new Set())
export const hiddenEdgeTypes = writable<Set<string>>(new Set())

export function toggleNodeType(type: string, visible: boolean): void {
  hiddenNodeTypes.update(s => {
    const n = new Set(s)
    if (visible) n.delete(type)
    else n.add(type)
    return n
  })
}

export function toggleEdgeType(type: string, visible: boolean): void {
  hiddenEdgeTypes.update(s => {
    const n = new Set(s)
    if (visible) n.delete(type)
    else n.add(type)
    return n
  })
}

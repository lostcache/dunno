<script lang="ts">
  import { get } from 'svelte/store'
  import { cyInstance, editingNode, hoverNode } from '../stores/graphStore'
  import { apiDel } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import { Button } from '$lib/components/ui/button'

  interface Item { id: string; name?: string; title?: string; content?: string }

  let { items, tab, onRefresh }: { items: Item[]; tab: string; onRefresh: () => void } = $props()

  function getLabel(item: Item): string {
    const raw = item.name || item.title || item.content || item.id || '?'
    return raw.length > 28 ? raw.slice(0, 28) + '…' : raw
  }

  function onMouseEnter(id: string) {
    const cy = get(cyInstance)
    if (!cy) return
    const node = cy.getElementById(id)
    if (!node || !node.length) return
    hoverNode.set(node.data())
  }

  function onMouseLeave() {
    // hoverNode cleared by NodeHoverBtns timeout
  }

  function onSelect(id: string) {
    const cy = get(cyInstance)
    if (!cy) return
    cy.$('node:selected').unselect()
    const node = cy.getElementById(id)
    if (!node || !node.length) return
    node.select()
    cy.animate({ center: { eles: node }, duration: 300 })
  }

  const DELETE_ENDPOINTS: Record<string, string> = {
    projects: '/api/projects',
    modules: '/api/modules',
    submodules: '/api/submodules',
    files: '/api/files',
    tasks: '/api/tasks',
    todos: '/api/todos',
    'user-stories': '/api/user-stories',
    epics: '/api/epics',
    personas: '/api/personas',
    workflows: '/api/workflows',
  }

  async function deleteItem(id: string, e: MouseEvent) {
    e.stopPropagation()
    const base = DELETE_ENDPOINTS[tab]
    if (!base) return
    try {
      await apiDel(`${base}/${encodeURIComponent(id)}`)
      setStatus(`Deleted ${id}`, 'ok')
      onRefresh()
    } catch (err: unknown) {
      setStatus('Delete failed: ' + (err as Error).message, 'err')
    }
  }
</script>

{#if items.length === 0}
  <div class="px-2 py-2 text-[#64748b] text-xs">No items</div>
{:else}
  {#each items as item}
    <div
      class="group flex items-center justify-between px-2 py-[5px] rounded cursor-pointer text-[#cbd5e1] text-xs hover:bg-[#1e2135]"
      role="button"
      tabindex="0"
      onmouseenter={() => onMouseEnter(item.id)}
      onmouseleave={onMouseLeave}
      onclick={() => onSelect(item.id)}
      onkeydown={(e) => e.key === 'Enter' && onSelect(item.id)}
    >
      <span title={item.id}>{getLabel(item)}</span>
      <Button
        variant="ghost"
        size="icon"
        class="h-5 w-5 opacity-0 group-hover:opacity-100 text-red-400 hover:text-red-400 hover:bg-transparent"
        onclick={(e: MouseEvent) => deleteItem(item.id, e)}
      >✕</Button>
    </div>
  {/each}
{/if}

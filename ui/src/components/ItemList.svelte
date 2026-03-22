<script lang="ts">
  import { get } from 'svelte/store'
  import { cyInstance, editingNode, hoverNode } from '../stores/graphStore'
  import { apiDel } from '../lib/api'
  import { setStatus } from '../stores/statusStore'

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
  <div class="empty">No items</div>
{:else}
  {#each items as item}
    <div
      class="item-row"
      role="button"
      tabindex="0"
      onmouseenter={() => onMouseEnter(item.id)}
      onmouseleave={onMouseLeave}
      onclick={() => onSelect(item.id)}
      onkeydown={(e) => e.key === 'Enter' && onSelect(item.id)}
    >
      <span title={item.id}>{getLabel(item)}</span>
      <button class="del-btn" onclick={(e) => deleteItem(item.id, e)}>✕</button>
    </div>
  {/each}
{/if}

<style>
  .empty { padding: 8px; color: #64748b; font-size: 12px; }
  .item-row {
    padding: 5px 8px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: #cbd5e1;
    font-size: 12px;
  }
  .item-row:hover { background: #1e2135; }
  .del-btn {
    opacity: 0;
    color: #f87171;
    font-size: 11px;
    border: none;
    background: none;
    cursor: pointer;
  }
  .item-row:hover .del-btn { opacity: 1; }
</style>

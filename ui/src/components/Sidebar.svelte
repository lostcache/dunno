<script lang="ts">
  import { activeTab, projectId } from '../stores/appStore'
  import { openCreate } from '../stores/modalStore'
  import { api } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import { ENTITY_TABS } from '../lib/constants'
  import ItemList from './ItemList.svelte'

  let { onRefresh }: { onRefresh: () => void } = $props()

  let items = $state<unknown[]>([])

  async function loadItems() {
    const tab = $activeTab
    const pid = $projectId
    let url: string | null = null

    if (tab === 'projects') url = '/api/projects'
    else if (tab === 'modules' && pid) url = `/api/projects/${pid}/modules`
    else if (tab === 'modules') url = '/api/modules'
    else if (tab === 'files' && pid) url = `/api/projects/${pid}/files`
    else if (tab === 'tasks' && pid) url = `/api/projects/${pid}/tasks`
    else if (tab === 'todos' && pid) url = `/api/projects/${pid}/todos`
    else if (tab === 'user-stories' && pid) url = `/api/projects/${pid}/user-stories`
    else if (tab === 'epics' && pid) url = `/api/projects/${pid}/epics`
    else if (tab === 'personas' && pid) url = `/api/projects/${pid}/personas`
    else if (tab === 'workflows' && pid) url = `/api/projects/${pid}/workflows`

    if (!url) { items = []; return }
    try {
      items = await api<unknown[]>(url)
    } catch (e: unknown) {
      setStatus('Load failed: ' + (e as Error).message, 'err')
    }
  }

  $effect(() => {
    $activeTab
    $projectId
    loadItems()
  })

  export { loadItems }
</script>

<aside>
  <div class="entity-tabs">
    {#each ENTITY_TABS as tab}
      <div
        class="entity-tab {tab === $activeTab ? 'active' : ''}"
        role="button"
        tabindex="0"
        onclick={() => activeTab.set(tab)}
        onkeydown={(e) => e.key === 'Enter' && activeTab.set(tab)}
      >{tab}</div>
    {/each}
  </div>
  <div class="item-list">
    {#if !$projectId && $activeTab !== 'projects'}
      <div class="no-project">Select a project</div>
    {:else}
      <ItemList items={items as any[]} tab={$activeTab} {onRefresh} />
    {/if}
  </div>
  <div class="sidebar-actions">
    <button onclick={() => openCreate($activeTab)}>+ Create</button>
  </div>
</aside>

<style>
  aside {
    width: 220px;
    flex-shrink: 0;
    background: #14172a;
    border-right: 1px solid #2d3148;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .entity-tabs {
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    flex: 1;
    padding: 8px 0;
  }
  .entity-tab {
    padding: 6px 16px;
    cursor: pointer;
    color: #94a3b8;
    font-size: 13px;
    border-left: 3px solid transparent;
  }
  .entity-tab:hover { background: #1e2135; color: #e2e8f0; }
  .entity-tab.active { border-left-color: #7c6df0; color: #a78bfa; background: #1e2135; }
  .item-list { padding: 0 8px 8px; overflow-y: auto; max-height: 280px; }
  .no-project { padding: 8px; color: #64748b; font-size: 12px; }
  .sidebar-actions { padding: 10px 12px; border-top: 1px solid #2d3148; display: flex; gap: 6px; }
  .sidebar-actions button {
    flex: 1;
    padding: 6px;
    background: #5b45d6;
    color: #fff;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }
  .sidebar-actions button:hover { background: #7c6df0; }
</style>

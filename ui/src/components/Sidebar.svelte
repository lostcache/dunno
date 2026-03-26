<script lang="ts">
  import { activeTab, projectId } from '../stores/appStore'
  import { openCreate } from '../stores/modalStore'
  import { api } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import { ENTITY_TABS } from '../lib/constants'
  import { Button } from '$lib/components/ui/button'
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
    else if (tab === 'issues') url = '/api/issues'

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

<aside class="w-[220px] shrink-0 bg-[#14172a] border-r border-[#2d3148] flex flex-col overflow-hidden">
  <div class="flex flex-col overflow-y-auto flex-1 py-2">
    {#each ENTITY_TABS as tab}
      <button
        class="px-4 py-1.5 cursor-pointer text-[13px] border-l-[3px] text-left transition-colors
          {tab === $activeTab
            ? 'border-[#7c6df0] text-[#a78bfa] bg-[#1e2135]'
            : 'border-transparent text-[#94a3b8] hover:bg-[#1e2135] hover:text-[#e2e8f0]'}"
        onclick={() => activeTab.set(tab)}
      >{tab}</button>
    {/each}
  </div>
  <div class="px-2 pb-2 overflow-y-auto max-h-[280px]">
    {#if !$projectId && $activeTab !== 'projects'}
      <div class="px-2 py-2 text-[#64748b] text-xs">Select a project</div>
    {:else}
      <ItemList items={items as any[]} tab={$activeTab} {onRefresh} />
    {/if}
  </div>
  <div class="px-3 py-2.5 border-t border-[#2d3148]">
    <Button
      class="w-full text-xs h-8"
      onclick={() => openCreate($activeTab)}
    >+ Create</Button>
  </div>
</aside>

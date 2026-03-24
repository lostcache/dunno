<script lang="ts">
  import { projectId, mainView, filterPanelOpen } from '../stores/appStore'
  import { api } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import { Button } from '$lib/components/ui/button'
  import * as Select from '$lib/components/ui/select'

  interface Project { id: string; name: string }

  let projects = $state<Project[]>([])

  export async function loadProjects() {
    try {
      const list = await api<Project[]>('/api/projects')
      projects = list
      if (!$projectId && list.length) {
        projectId.set(list[0].id)
      }
    } catch (e: unknown) {
      setStatus('Failed to load projects: ' + (e as Error).message, 'err')
    }
  }

  let selectedProjectName = $derived(
    $projectId
      ? (projects.find(p => p.id === $projectId)?.name ?? $projectId)
      : '— select project —'
  )
</script>

<nav class="flex items-center gap-3 px-4 py-2 bg-[#1a1d27] border-b border-[#2d3148] flex-shrink-0">
  <h1 class="text-base font-bold text-[#a78bfa] mr-2">Dunno UI</h1>

  <Select.Root type="single" value={$projectId ?? ''} onValueChange={(v) => projectId.set(v || null)}>
    <Select.Trigger class="h-8 bg-[#252840] border-[#3d4165] text-[#e2e8f0] hover:bg-[#3d4165] text-xs w-48">
      {selectedProjectName}
    </Select.Trigger>
    <Select.Content>
      {#each projects as p}
        <Select.Item value={p.id} label={p.name} />
      {/each}
    </Select.Content>
  </Select.Root>

  <Button
    variant="ghost"
    size="sm"
    class="h-8 px-3 bg-[#252840] border border-[#3d4165] text-[#e2e8f0] hover:bg-[#3d4165] text-base"
    onclick={() => document.dispatchEvent(new CustomEvent('dunno:refresh'))}
    title="Refresh"
  >↻</Button>

  <Button
    variant="ghost"
    size="sm"
    class="h-8 px-3 border border-[#3d4165] text-[#e2e8f0] hover:bg-[#3d4165] text-xs {$filterPanelOpen ? 'bg-[#5b45d6] border-[#7c6df0]' : 'bg-[#252840]'}"
    onclick={() => filterPanelOpen.update(v => !v)}
  >Filter</Button>

  <div class="ml-auto flex gap-1.5">
    <Button
      variant={$mainView === 'graph' ? 'default' : 'outline'}
      size="sm"
      class="h-8 px-3 text-xs {$mainView !== 'graph' ? 'border-[#3d4165] text-[#e2e8f0] bg-[#252840] hover:bg-[#3d4165]' : ''}"
      onclick={() => mainView.set('graph')}
    >Graph</Button>
    <Button
      variant={$mainView === 'ctx' ? 'default' : 'outline'}
      size="sm"
      class="h-8 px-3 text-xs {$mainView !== 'ctx' ? 'border-[#3d4165] text-[#e2e8f0] bg-[#252840] hover:bg-[#3d4165]' : ''}"
      onclick={() => mainView.set('ctx')}
    >Context</Button>
  </div>
</nav>

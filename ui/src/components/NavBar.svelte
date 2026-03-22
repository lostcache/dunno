<script lang="ts">
  import { projectId, mainView, filterPanelOpen } from '../stores/appStore'
  import { api } from '../lib/api'
  import { setStatus } from '../stores/statusStore'

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

  function onProjectChange(e: Event) {
    const val = (e.target as HTMLSelectElement).value
    projectId.set(val || null)
  }
</script>

<nav>
  <h1>Dunno UI</h1>
  <select onchange={onProjectChange} value={$projectId ?? ''}>
    <option value="">— select project —</option>
    {#each projects as p}
      <option value={p.id}>{p.name}</option>
    {/each}
  </select>
  <button onclick={() => document.dispatchEvent(new CustomEvent('dunno:refresh'))} title="Refresh">↻</button>
  <button
    id="btn-filter"
    class={$filterPanelOpen ? 'active' : ''}
    onclick={() => filterPanelOpen.update(v => !v)}
  >Filter</button>
  <div class="view-btns">
    <button
      class={$mainView === 'graph' ? 'active' : ''}
      onclick={() => mainView.set('graph')}
    >Graph</button>
    <button
      class={$mainView === 'ctx' ? 'active' : ''}
      onclick={() => mainView.set('ctx')}
    >Context</button>
  </div>
</nav>

<style>
  nav {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    background: #1a1d27;
    border-bottom: 1px solid #2d3148;
    flex-shrink: 0;
  }
  nav h1 { font-size: 16px; font-weight: 700; color: #a78bfa; margin-right: 8px; }
  nav select {
    background: #252840;
    color: #e2e8f0;
    border: 1px solid #3d4165;
    padding: 4px 8px;
    border-radius: 4px;
  }
  nav button {
    background: #252840;
    color: #e2e8f0;
    border: 1px solid #3d4165;
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
  }
  nav button:hover { background: #3d4165; }
  .view-btns { margin-left: auto; display: flex; gap: 6px; }
  .view-btns button.active,
  #btn-filter.active { background: #5b45d6; border-color: #7c6df0; }
</style>

<script lang="ts">
  import { onMount } from 'svelte'
  import { mainView } from './stores/appStore'
  import { setStatus } from './stores/statusStore'
  import NavBar from './components/NavBar.svelte'
  import FilterPanel from './components/FilterPanel.svelte'
  import Sidebar from './components/Sidebar.svelte'
  import GraphView from './components/GraphView.svelte'
  import ContextView from './components/ContextView.svelte'
  import StatusBar from './components/StatusBar.svelte'
  import Modal from './components/Modal.svelte'

  let navbar = $state<ReturnType<typeof NavBar> | null>(null)
  let sidebar = $state<ReturnType<typeof Sidebar> | null>(null)
  let graphView = $state<ReturnType<typeof GraphView> | null>(null)

  async function refresh() {
    if (navbar) await navbar.loadProjects()
    if (graphView) await graphView.reload()
    if (sidebar) await sidebar.loadItems()
    setStatus('Refreshed', 'ok')
  }

  onMount(async () => {
    if (navbar) await navbar.loadProjects()
    if (graphView) await graphView.reload()
    if (sidebar) await sidebar.loadItems()

    document.addEventListener('dunno:refresh', refresh)
    return () => document.removeEventListener('dunno:refresh', refresh)
  })
</script>

<NavBar bind:this={navbar} />
<FilterPanel />

<div class="main-area">
  <Sidebar bind:this={sidebar} onRefresh={refresh} />
  <main>
    {#if $mainView === 'graph'}
      <GraphView bind:this={graphView} onRefresh={refresh} />
    {:else}
      <ContextView />
    {/if}
  </main>
</div>

<StatusBar />
<Modal onRefresh={refresh} />

<style>
  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    font-family: system-ui, sans-serif;
    font-size: 14px;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: #0f1117;
    color: #e2e8f0;
  }
  :global(#app) { display: flex; flex-direction: column; height: 100vh; }
  .main-area { display: flex; flex: 1; overflow: hidden; }
  main { flex: 1; overflow: hidden; display: flex; flex-direction: column; }
</style>

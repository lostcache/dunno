<script lang="ts">
  import { onMount, onDestroy } from 'svelte'
  import { get } from 'svelte/store'
  import cytoscape from 'cytoscape'
  import { projectId } from '../stores/appStore'
  import { cyInstance, editingNode } from '../stores/graphStore'
  import { hiddenNodeTypes, hiddenEdgeTypes } from '../stores/filterStore'
  import { buildCyStyles, applyFilters } from '../lib/cytoscapeHelpers'
  import { api } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import NodeDetail from './NodeDetail.svelte'
  import MultiSelectPanel from './MultiSelectPanel.svelte'
  import NodeHoverBtns from './NodeHoverBtns.svelte'
  import type { NodeData } from '../lib/types'

  let { onRefresh }: { onRefresh: () => void } = $props()

  let cyContainer = $state<HTMLDivElement | null>(null)
  let graphView = $state<HTMLDivElement | null>(null)
  let cy: cytoscape.Core | null = null
  let hoverBtns = $state<ReturnType<typeof NodeHoverBtns> | null>(null)
  let selectedNodes = $state<NodeData[]>([])
  let hoverHideTimer: ReturnType<typeof setTimeout> | null = null

  async function loadGraph(pid?: string | null) {
    if (!cy) return
    try {
      const url = pid ? `/api/projects/${pid}/graph` : '/api/graph'
      const data = await api<{ elements: unknown[] }>(url)
      cy.elements().remove()
      cy.add(data.elements as cytoscape.ElementDefinition[])
      cy.layout({ name: 'cose', animate: false, randomize: true } as cytoscape.LayoutOptions).run()
      applyFilters(cy, get(hiddenNodeTypes), get(hiddenEdgeTypes))
    } catch (e: unknown) {
      setStatus('Graph load failed: ' + (e as Error).message, 'err')
    }
  }

  export async function reload() {
    await loadGraph(get(projectId))
  }

  function onSelectionChange() {
    if (!cy) return
    const selected = cy.$('node:selected')
    if (selected.length >= 2) {
      editingNode.set(null)
      selectedNodes = selected.map((n: any) => n.data())
    } else if (selected.length === 1) {
      selectedNodes = []
      editingNode.set(selected[0].data())
    } else {
      selectedNodes = []
      editingNode.set(null)
    }
  }

  onMount(() => {
    if (!cyContainer) return

    cy = cytoscape({
      container: cyContainer,
      style: buildCyStyles(),
      layout: { name: 'cose' } as cytoscape.LayoutOptions,
      wheelSensitivity: 0.3,
      boxSelectionEnabled: true,
    })

    cyInstance.set(cy)

    cy.on('tap', 'node', () => {
      setTimeout(() => onSelectionChange(), 0)
    })
    cy.on('select unselect', 'node', () => {
      setTimeout(() => onSelectionChange(), 0)
    })
    cy.on('tap', (evt: cytoscape.EventObject) => {
      if (evt.target === cy) {
        cy!.$('node:selected').unselect()
        editingNode.set(null)
        selectedNodes = []
      }
    })
    cy.on('mouseover', 'node', (evt: cytoscape.EventObject) => {
      if (hoverHideTimer) { clearTimeout(hoverHideTimer); hoverHideTimer = null }
      const node = evt.target as cytoscape.NodeSingular
      if (hoverBtns) hoverBtns.showAt(node.data(), node.renderedPosition())
    })
    cy.on('mouseout', 'node', () => {
      if (hoverBtns) hoverBtns.scheduleHide()
    })

    loadGraph(get(projectId))
  })

  onDestroy(() => {
    if (cy) { cy.destroy(); cy = null }
    cyInstance.set(null)
  })

  $effect(() => {
    const pid = $projectId
    if (cy) loadGraph(pid)
  })

  $effect(() => {
    const hn = $hiddenNodeTypes
    const he = $hiddenEdgeTypes
    if (cy) applyFilters(cy, hn, he)
  })
</script>

<div id="graph-view" bind:this={graphView}>
  <div id="cy" bind:this={cyContainer}></div>

  <NodeHoverBtns
    bind:this={hoverBtns}
    cyContainer={cyContainer}
    graphView={graphView}
    {onRefresh}
  />

  {#if selectedNodes.length >= 2}
    <MultiSelectPanel nodes={selectedNodes} {onRefresh} />
  {:else if $editingNode}
    <NodeDetail />
  {/if}
</div>

<style>
  #graph-view { flex: 1; display: flex; flex-direction: column; position: relative; }
  #cy { flex: 1; background: #0f1117; }
</style>

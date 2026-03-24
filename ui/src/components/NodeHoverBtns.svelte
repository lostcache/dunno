<script lang="ts">
  import { hoverNode, editingNode } from '../stores/graphStore'
  import { openEdit } from '../stores/modalStore'
  import { apiDel } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import { EDIT_ENDPOINTS } from '../lib/constants'
  import { Button } from '$lib/components/ui/button'

  let { cyContainer, graphView, onRefresh }: {
    cyContainer: HTMLDivElement | null
    graphView: HTMLDivElement | null
    onRefresh: () => void
  } = $props()

  let visible = $state(false)
  let left = $state(0)
  let top = $state(0)
  let hideTimer: ReturnType<typeof setTimeout> | null = null

  export function showAt(nodeData: import('../lib/types').NodeData, renderedPos: { x: number; y: number }) {
    if (!cyContainer || !graphView) return
    clearHideTimer()
    const cyRect = cyContainer.getBoundingClientRect()
    const gRect = graphView.getBoundingClientRect()
    left = cyRect.left - gRect.left + renderedPos.x
    top = cyRect.top - gRect.top + renderedPos.y
    editingNode.set(nodeData)
    hoverNode.set(nodeData)
    visible = true
  }

  export function scheduleHide() {
    hideTimer = setTimeout(() => { visible = false }, 200)
  }

  function clearHideTimer() {
    if (hideTimer) { clearTimeout(hideTimer); hideTimer = null }
  }

  async function deleteNode() {
    const data = $hoverNode
    if (!data) return
    const base = EDIT_ENDPOINTS[data.node_type]
    if (!base) { setStatus('No delete endpoint for ' + data.node_type, 'err'); return }
    visible = false
    try {
      await apiDel(`${base}/${encodeURIComponent(data.id)}`)
      editingNode.set(null)
      hoverNode.set(null)
      setStatus('Deleted ' + data.label, 'ok')
      onRefresh()
    } catch (e: unknown) {
      setStatus('Delete failed: ' + (e as Error).message, 'err')
    }
  }
</script>

{#if visible && $hoverNode}
  <div
    id="node-hover-btns"
    style="left:{left}px;top:{top}px"
    onmouseenter={clearHideTimer}
    onmouseleave={scheduleHide}
    role="toolbar"
    tabindex="0"
  >
    <Button size="sm" onclick={() => { if ($hoverNode) openEdit($hoverNode) }}>Edit</Button>
    <Button size="sm" variant="destructive" onclick={deleteNode}>Delete</Button>
  </div>
{/if}

<style>
  #node-hover-btns {
    position: absolute;
    display: flex;
    gap: 4px;
    z-index: 20;
    pointer-events: auto;
    transform: translate(-50%, calc(-100% - 8px));
  }
</style>

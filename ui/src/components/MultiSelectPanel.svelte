<script lang="ts">
  import { get } from 'svelte/store'
  import { cyInstance } from '../stores/graphStore'
  import { apiPost } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import { findEdgePair } from '../lib/constants'
  import type { NodeData } from '../lib/types'

  let { nodes, onRefresh }: { nodes: NodeData[]; onRefresh: () => void } = $props()

  interface PairRow {
    lines: string[]
    noPair: boolean
  }

  let pairs = $derived<PairRow[]>(() => {
    const rows: PairRow[] = []
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i], b = nodes[j]
        const pair = findEdgePair(a.node_type, b.node_type)
        if (pair) {
          const [src, dst] = pair.a === a.node_type ? [a, b] : [b, a]
          rows.push({
            lines: [
              `${src.label} → ${pair.a_to_b} → ${dst.label}`,
              `${dst.label} → ${pair.b_to_a} → ${src.label}`,
            ],
            noPair: false,
          })
        } else if (a.node_type === 'context' || b.node_type === 'context') {
          const [nonCtx, ctx] = a.node_type === 'context' ? [b, a] : [a, b]
          rows.push({ lines: [`${nonCtx.label} → has_context → ${ctx.label}`], noPair: false })
        } else {
          rows.push({ lines: [`${a.label} ↔ ${b.label}: no known edge pair`], noPair: true })
        }
      }
    }
    return rows
  })

  async function linkSelected() {
    const cy = get(cyInstance)
    if (!cy) return
    const selected = cy.$('node:selected').map((n: any) => n.data()) as NodeData[]
    let linked = 0
    const errors: string[] = []
    for (let i = 0; i < selected.length; i++) {
      for (let j = i + 1; j < selected.length; j++) {
        const a = selected[i], b = selected[j]
        const pair = findEdgePair(a.node_type, b.node_type)
        if (pair) {
          const [srcId, dstId] = pair.a === a.node_type ? [a.id, b.id] : [b.id, a.id]
          try {
            await apiPost('/api/link', { from_id: srcId, edge: pair.a_to_b, to_id: dstId })
            await apiPost('/api/link', { from_id: dstId, edge: pair.b_to_a, to_id: srcId })
            linked++
          } catch (e: unknown) { errors.push((e as Error).message) }
        } else if (a.node_type === 'context' || b.node_type === 'context') {
          const [nonCtx, ctx] = a.node_type === 'context' ? [b, a] : [a, b]
          try {
            await apiPost('/api/link', { from_id: nonCtx.id, edge: 'has_context', to_id: ctx.id })
            linked++
          } catch (e: unknown) { errors.push((e as Error).message) }
        }
      }
    }
    if (errors.length) {
      setStatus(`Linked ${linked} pair(s), ${errors.length} error(s): ${errors[0]}`, 'err')
    } else if (linked > 0) {
      setStatus(`Linked ${linked} pair(s)`, 'ok')
      onRefresh()
    } else {
      setStatus('No linkable pairs found', 'err')
    }
  }
</script>

{#if nodes.length >= 2}
  <div id="multi-select-panel">
    <div class="msp-header">
      <h3>{nodes.length} nodes selected</h3>
      <button onclick={linkSelected}>Link</button>
    </div>
    <div class="msp-pairs">
      {#each pairs as row}
        <div class="pair-row {row.noPair ? 'no-pair' : ''}">
          {#each row.lines as line}
            <div>{line}</div>
          {/each}
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
  #multi-select-panel {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 10px 16px;
    background: #14172a;
    border-top: 1px solid #2d3148;
    max-height: 200px;
    overflow-y: auto;
    font-size: 12px;
    z-index: 10;
  }
  #multi-select-panel h3 { color: #a78bfa; }
  .msp-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px; }
  .msp-header button { padding: 3px 10px; background: #16a34a; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
  .msp-pairs { margin-top: 8px; }
  .pair-row { padding: 4px 0; border-bottom: 1px solid #2d3148; color: #94a3b8; font-size: 11px; }
  .pair-row:last-child { border-bottom: none; }
  .pair-row.no-pair { color: #475569; }
</style>

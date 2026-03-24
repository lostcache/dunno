<script lang="ts">
  import { get } from 'svelte/store'
  import { cyInstance } from '../stores/graphStore'
  import { apiPost } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import { findEdgePair } from '../lib/constants'
  import type { NodeData } from '../lib/types'
  import { Button } from '$lib/components/ui/button'

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
  <div class="absolute bottom-0 inset-x-0 px-4 py-[10px] bg-[#14172a] border-t border-[#2d3148] max-h-[200px] overflow-y-auto text-xs z-10">
    <div class="flex items-center justify-between mb-1">
      <h3 class="text-[#a78bfa]">{nodes.length} nodes selected</h3>
      <Button size="sm" class="bg-green-700 hover:bg-green-600 text-white h-6 px-2.5 text-xs" onclick={linkSelected}>Link</Button>
    </div>
    <div class="mt-2">
      {#each pairs as row}
        <div class="py-1 border-b border-[#2d3148] last:border-b-0 text-[11px] {row.noPair ? 'text-[#475569]' : 'text-[#94a3b8]'}">
          {#each row.lines as line}
            <div>{line}</div>
          {/each}
        </div>
      {/each}
    </div>
  </div>
{/if}

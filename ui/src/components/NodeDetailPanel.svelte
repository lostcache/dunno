<script lang="ts">
  import { editingNode } from '../stores/graphStore'
  import { openEdit, openAddLink } from '../stores/modalStore'
  import { Button } from '$lib/components/ui/button'
  import { Badge } from '$lib/components/ui/badge'
  import { Separator } from '$lib/components/ui/separator'
  import { ScrollArea } from '$lib/components/ui/scroll-area'
  import { Card, CardHeader, CardTitle, CardContent } from '$lib/components/ui/card'

  function toTitleCase(key: string): string {
    return key.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
  }

  function getDisplayFields(node: Record<string, unknown>): [string, unknown][] {
    return Object.entries(node).filter(([k, v]) => k !== 'id' && k !== 'label' && k !== 'node_type' && v !== node.label)
  }
</script>

{#if $editingNode}
  <div class="w-72 flex flex-col border-l border-[#2d3148] bg-[#14172a] overflow-hidden shrink-0">
    <Card class="rounded-none border-0 bg-transparent flex flex-col h-full">
      <CardHeader class="pb-2 pt-3 px-3 gap-1.5">
        <div class="flex items-start justify-between gap-2">
          <div class="flex flex-col gap-1 min-w-0">
            <Badge variant="secondary" class="w-fit text-[10px]">{$editingNode.node_type}</Badge>
            <CardTitle class="text-sm text-[#a78bfa] leading-tight break-words">{$editingNode.label}</CardTitle>
          </div>
          <div class="flex flex-col gap-1 shrink-0">
            <Button size="sm" class="h-6 px-2 text-xs" onclick={() => openEdit($editingNode!)}>Edit</Button>
            <Button size="sm" class="h-6 px-2 text-xs bg-green-700 hover:bg-green-600 text-white" onclick={() => openAddLink($editingNode!)}>Add &amp; Link</Button>
          </div>
        </div>
      </CardHeader>

      <Separator class="bg-[#2d3148]" />

      <CardContent class="flex-1 p-0 overflow-hidden">
        <ScrollArea class="h-full px-3 py-2">
          <div class="flex flex-col gap-2">
            {#each getDisplayFields($editingNode) as [key, value]}
              <div class="flex flex-col gap-0.5">
                <span class="text-[10px] text-[#64748b] uppercase tracking-wide">{toTitleCase(key)}</span>
                {#if Array.isArray(value)}
                  <div class="flex flex-wrap gap-1">
                    {#each value as item}
                      <Badge variant="outline" class="text-[10px]">{item}</Badge>
                    {/each}
                  </div>
                {:else if typeof value === 'boolean'}
                  <span class="text-xs text-[#94a3b8]">{value ? '✓' : '✗'}</span>
                {:else if value !== null && value !== undefined && value !== ''}
                  <span class="text-xs text-[#94a3b8] break-words">{value}</span>
                {:else}
                  <span class="text-xs text-[#475569] italic">—</span>
                {/if}
              </div>
            {/each}
          </div>

          <div class="mt-3 pt-2 border-t border-[#2d3148]">
            <span class="text-[9px] text-[#475569] font-mono break-all">{$editingNode.id}</span>
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  </div>
{/if}

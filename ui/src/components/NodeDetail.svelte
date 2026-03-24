<script lang="ts">
  import { editingNode } from '../stores/graphStore'
  import { openEdit, openAddLink } from '../stores/modalStore'
  import { Button } from '$lib/components/ui/button'
</script>

{#if $editingNode}
  <div class="absolute bottom-0 inset-x-0 px-4 py-[10px] bg-[#14172a] border-t border-[#2d3148] max-h-40 overflow-y-auto text-xs z-10">
    <div class="flex items-center justify-between mb-1">
      <h3 class="text-[#a78bfa]">{$editingNode.node_type}: {$editingNode.label}</h3>
      <div class="flex gap-1.5">
        <Button size="sm" class="bg-green-700 hover:bg-green-600 text-white h-6 px-2.5 text-xs" onclick={() => openAddLink($editingNode!)}>Add &amp; Link</Button>
        <Button size="sm" onclick={() => openEdit($editingNode!)}>Edit</Button>
      </div>
    </div>
    <pre class="whitespace-pre-wrap break-all text-[#94a3b8]">{JSON.stringify($editingNode, null, 2)}</pre>
  </div>
{/if}

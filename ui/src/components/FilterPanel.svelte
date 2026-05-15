<script lang="ts">
  import { filterPanelOpen } from "../stores/appStore";
  import {
    hiddenNodeTypes,
    hiddenEdgeTypes,
    toggleNodeType,
    toggleEdgeType,
  } from "../stores/filterStore";
  import { NODE_COLORS, ALL_EDGE_TYPES } from "../lib/constants";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Label } from "$lib/components/ui/label";
</script>

{#if $filterPanelOpen}
  <div class="flex flex-wrap gap-6 px-4 py-2 border-b border-[#2d3148] bg-[#1a1d27] flex-shrink-0">
    <div class="flex flex-wrap gap-2 items-center">
      <strong class="mr-1 text-[11px] text-[#94a3b8] whitespace-nowrap">Nodes</strong>
      {#each Object.entries(NODE_COLORS) as [type, color]}
        <Label
          class="flex items-center gap-1 text-[11px] cursor-pointer text-[#cbd5e1] select-none font-normal"
        >
          <Checkbox
            checked={!$hiddenNodeTypes.has(type)}
            onCheckedChange={(v) => toggleNodeType(type, v as boolean)}
            class="size-3 rounded-none"
          />
          <span
            class="w-2.5 h-2.5 rounded-sm inline-block flex-shrink-0"
            style="background:{color.bg}"
          ></span>
          {type}
        </Label>
      {/each}
    </div>
    <div class="flex flex-wrap gap-2 items-center">
      <strong class="mr-1 text-[11px] text-[#94a3b8] whitespace-nowrap">Edges</strong>
      {#each ALL_EDGE_TYPES as type}
        <Label
          class="flex items-center gap-1 text-[11px] cursor-pointer text-[#cbd5e1] select-none font-normal"
        >
          <Checkbox
            checked={!$hiddenEdgeTypes.has(type)}
            onCheckedChange={(v) => toggleEdgeType(type, v as boolean)}
            class="size-3 rounded-none"
          />
          {type}
        </Label>
      {/each}
    </div>
  </div>
{/if}

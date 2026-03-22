<script lang="ts">
  import { filterPanelOpen } from '../stores/appStore'
  import { toggleNodeType, toggleEdgeType } from '../stores/filterStore'
  import { NODE_COLORS, ALL_EDGE_TYPES } from '../lib/constants'
</script>

{#if $filterPanelOpen}
  <div id="filter-panel">
    <div class="filter-section">
      <strong>Nodes</strong>
      {#each Object.entries(NODE_COLORS) as [type, color]}
        <label class="filter-checkbox">
          <input
            type="checkbox"
            checked
            onchange={(e) => toggleNodeType(type, (e.target as HTMLInputElement).checked)}
          />
          <span class="filter-dot" style="background:{color.bg}"></span>
          {type}
        </label>
      {/each}
    </div>
    <div class="filter-section">
      <strong>Edges</strong>
      {#each ALL_EDGE_TYPES as type}
        <label class="filter-checkbox">
          <input
            type="checkbox"
            checked
            onchange={(e) => toggleEdgeType(type, (e.target as HTMLInputElement).checked)}
          />
          {type}
        </label>
      {/each}
    </div>
  </div>
{/if}

<style>
  #filter-panel {
    background: #1a1d27;
    border-bottom: 1px solid #2d3148;
    padding: 8px 16px;
    display: flex;
    gap: 24px;
    flex-wrap: wrap;
    flex-shrink: 0;
  }
  .filter-section { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
  .filter-section strong { margin-right: 4px; font-size: 11px; color: #94a3b8; white-space: nowrap; }
  .filter-checkbox { display: flex; align-items: center; gap: 4px; font-size: 11px; cursor: pointer; color: #cbd5e1; user-select: none; }
  .filter-checkbox input { cursor: pointer; }
  .filter-dot { width: 10px; height: 10px; border-radius: 2px; display: inline-block; flex-shrink: 0; }
</style>

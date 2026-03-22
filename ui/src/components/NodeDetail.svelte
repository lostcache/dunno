<script lang="ts">
  import { editingNode } from '../stores/graphStore'
  import { openEdit, openAddLink } from '../stores/modalStore'
</script>

{#if $editingNode}
  <div id="node-detail">
    <div class="nd-header">
      <h3>{$editingNode.node_type}: {$editingNode.label}</h3>
      <div class="nd-actions">
        <button class="btn-add-link" onclick={() => openAddLink($editingNode!)}>Add &amp; Link</button>
        <button class="btn-edit" onclick={() => openEdit($editingNode!)}>Edit</button>
      </div>
    </div>
    <pre>{JSON.stringify($editingNode, null, 2)}</pre>
  </div>
{/if}

<style>
  #node-detail {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 10px 16px;
    background: #14172a;
    border-top: 1px solid #2d3148;
    max-height: 160px;
    overflow-y: auto;
    font-size: 12px;
    z-index: 10;
  }
  #node-detail h3 { color: #a78bfa; margin-bottom: 6px; }
  #node-detail pre { white-space: pre-wrap; word-break: break-all; color: #94a3b8; }
  .nd-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px; }
  .nd-actions { display: flex; gap: 6px; }
  .btn-add-link { padding: 3px 10px; background: #16a34a; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
  .btn-edit { padding: 3px 10px; background: #5b45d6; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
</style>

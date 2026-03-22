<script lang="ts">
  import { statusState } from '../stores/statusStore'
</script>

<div id="status" class={$statusState.cls}>{$statusState.message}</div>

<style>
  #status {
    padding: 5px 16px;
    background: #14172a;
    border-top: 1px solid #2d3148;
    font-size: 12px;
    color: #64748b;
    flex-shrink: 0;
    min-height: 24px;
  }
  #status:global(.ok) { color: #4ade80; }
  #status:global(.err) { color: #f87171; }
</style>

<script lang="ts">
  import { projectId } from '../stores/appStore'
  import { api } from '../lib/api'
  import { setStatus } from '../stores/statusStore'

  interface Task { id: string; name?: string; title?: string }

  let ctxType = $state<'task' | 'file' | 'epic'>('task')
  let ctxId = $state('')
  let ctxFull = $state(false)
  let ctxOutput = $state('Select type and enter an ID, then click Fetch.')
  let tasks = $state<Task[]>([])

  async function loadTasks() {
    const pid = $projectId
    if (!pid) { tasks = []; return }
    try {
      tasks = await api<Task[]>(`/api/projects/${pid}/tasks`)
    } catch {
      tasks = []
    }
  }

  $effect(() => {
    if (ctxType === 'task') {
      $projectId // track
      loadTasks()
    }
  })

  // Set ctxId to first task when tasks load
  $effect(() => {
    if (ctxType === 'task' && tasks.length > 0 && !ctxId) {
      ctxId = tasks[0].id
    }
  })

  async function fetchCtx() {
    if (!ctxId) { setStatus('Enter an ID', 'err'); return }
    try {
      const data = await api(`/api/ctx/${ctxType}/${encodeURIComponent(ctxId)}?full=${ctxFull}`)
      ctxOutput = JSON.stringify(data, null, 2)
      setStatus('Context fetched', 'ok')
    } catch (e: unknown) {
      setStatus('Fetch failed: ' + (e as Error).message, 'err')
    }
  }
</script>

<div id="ctx-view">
  <div class="ctx-controls">
    <select bind:value={ctxType} onchange={() => { ctxId = '' }}>
      <option value="task">Task</option>
      <option value="file">File</option>
      <option value="epic">Epic</option>
    </select>
    <div id="ctx-id-container">
      {#if ctxType === 'task' && tasks.length > 0}
        <select bind:value={ctxId}>
          {#each tasks as t}
            <option value={t.id}>{t.title || t.name || t.id}</option>
          {/each}
        </select>
      {:else}
        <input bind:value={ctxId} placeholder={ctxType === 'task' ? 'No tasks found' : 'record id e.g. task:abc123'} />
      {/if}
    </div>
    <label><input type="checkbox" bind:checked={ctxFull} /> Full</label>
    <button onclick={fetchCtx}>Fetch</button>
  </div>
  <div id="ctx-output">
    <pre>{ctxOutput}</pre>
  </div>
</div>

<style>
  #ctx-view {
    flex: 1;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 0;
    overflow: hidden;
  }
  .ctx-controls { display: flex; gap: 8px; align-items: center; }
  .ctx-controls select,
  .ctx-controls input {
    background: #252840;
    color: #e2e8f0;
    border: 1px solid #3d4165;
    padding: 6px 10px;
    border-radius: 4px;
  }
  #ctx-id-container { flex: 1; display: flex; }
  #ctx-id-container input,
  #ctx-id-container select {
    flex: 1;
    background: #252840;
    color: #e2e8f0;
    border: 1px solid #3d4165;
    padding: 6px 10px;
    border-radius: 4px;
  }
  .ctx-controls button { padding: 6px 14px; background: #5b45d6; color: #fff; border: none; border-radius: 4px; cursor: pointer; }
  .ctx-controls button:hover { background: #7c6df0; }
  .ctx-controls label { display: flex; align-items: center; gap: 4px; color: #94a3b8; }
  #ctx-output { flex: 1; overflow-y: auto; min-height: 0; }
  #ctx-output pre {
    background: #14172a;
    border: 1px solid #2d3148;
    padding: 12px;
    border-radius: 6px;
    white-space: pre-wrap;
    word-break: break-all;
    color: #94a3b8;
    font-size: 12px;
  }
</style>

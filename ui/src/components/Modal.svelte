<script lang="ts">
  import { get } from 'svelte/store'
  import { modalState, closeModal } from '../stores/modalStore'
  import { projectId } from '../stores/appStore'
  import { editingNode } from '../stores/graphStore'
  import { api, apiPost, apiPatch } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import {
    SCHEMAS, EDIT_SCHEMAS, EDIT_ENDPOINTS, CREATE_ENDPOINTS, TYPE_MAP,
    ALL_EDGE_TYPES, findEdgePair,
  } from '../lib/constants'
  import type { SchemaField } from '../lib/types'

  let { onRefresh }: { onRefresh: () => void } = $props()

  // For add-link mode: the type of new node to create
  let addLinkType = $state(Object.keys(CREATE_ENDPOINTS)[0])

  // Reactive schema for add-link sub-form
  let addLinkSchema = $derived.by<SchemaField[]>(() => {
    const schema = SCHEMAS[addLinkType]
    if (!schema) return []
    if (addLinkType === 'contexts') return schema.filter(f => f.name !== 'link_to')
    return schema
  })

  let addLinkEdgeInfo = $derived.by<{ fwd: string; rev: string } | { manual: true } | null>(() => {
    const node = $modalState.editingNode
    if (!node || addLinkType === 'contexts') return null
    const newNodeType = TYPE_MAP[addLinkType] || addLinkType
    const pair = findEdgePair(node.node_type, newNodeType)
    if (pair) {
      const [fwd, rev] = pair.a === node.node_type
        ? [pair.a_to_b, pair.b_to_a]
        : [pair.b_to_a, pair.a_to_b]
      return { fwd, rev }
    }
    return { manual: true }
  })

  let manualEdgeFwd = $state(ALL_EDGE_TYPES[0])

  // Form field values — keyed by field name
  let fieldValues = $state<Record<string, string>>({})

  function initFields() {
    const ms = get(modalState)
    const newValues: Record<string, string> = {}
    if (ms.mode === 'edit' && ms.editingNode) {
      const schema = EDIT_SCHEMAS[ms.editingNode.node_type] || []
      for (const f of schema) {
        newValues[f.name] = String(ms.editingNode[f.name] ?? '')
      }
      fieldValues = newValues
    } else if (ms.mode === 'create' && ms.tab) {
      const schema = SCHEMAS[ms.tab] || []
      const pid = get(projectId) || ''
      for (const f of schema) {
        newValues[f.name] = f.fill === 'projectId' ? pid : ''
      }
      fieldValues = newValues
    } else if (ms.mode === 'add-link') {
      addLinkType = Object.keys(CREATE_ENDPOINTS)[0]
      initAddLinkFields()
    }
  }

  function initAddLinkFields() {
    const pid = get(projectId) || ''
    const schema = addLinkSchema
    const newValues: Record<string, string> = {}
    for (const f of schema) {
      const key = 'field_' + f.name
      newValues[key] = f.fill === 'projectId' ? pid : ''
    }
    fieldValues = newValues
  }

  $effect(() => {
    if ($modalState.open) initFields()
  })

  $effect(() => {
    // Re-init add-link fields when type changes
    if ($modalState.mode === 'add-link') {
      addLinkType
      initAddLinkFields()
    }
  })

  async function submitCreate() {
    const tab = $modalState.tab!
    let url: string
    let body: unknown

    if (tab === 'contexts') {
      const fields: Record<string, string> = {}
      if (fieldValues.fields_type) fields.type = fieldValues.fields_type
      if (fieldValues.fields_content) fields.content = fieldValues.fields_content
      if (fieldValues.fields_description) fields.description = fieldValues.fields_description
      body = { fields, link_to: fieldValues.link_to }
      url = '/api/contexts'
    } else {
      url = CREATE_ENDPOINTS[tab]
      body = { ...fieldValues }
    }

    try {
      await apiPost(url, body)
      closeModal()
      setStatus('Created successfully', 'ok')
      onRefresh()
    } catch (e: unknown) {
      setStatus('Create failed: ' + (e as Error).message, 'err')
    }
  }

  async function submitEdit() {
    const node = $modalState.editingNode
    if (!node) return
    const endpoint = EDIT_ENDPOINTS[node.node_type]
    if (!endpoint) { setStatus('No endpoint for ' + node.node_type, 'err'); return }

    let body: unknown
    if (node.node_type === 'context') {
      body = { fields: { ...fieldValues } }
    } else {
      body = { ...fieldValues }
    }

    try {
      await apiPatch(`${endpoint}/${encodeURIComponent(node.id)}`, body)
      closeModal()
      editingNode.set(null)
      setStatus('Saved', 'ok')
      onRefresh()
    } catch (e: unknown) {
      setStatus('Save failed: ' + (e as Error).message, 'err')
    }
  }

  async function submitAddLink() {
    const node = $modalState.editingNode
    if (!node) return
    const endpoint = CREATE_ENDPOINTS[addLinkType]
    if (!endpoint) { setStatus('Unknown type: ' + addLinkType, 'err'); return }

    try {
      let body: unknown
      if (addLinkType === 'contexts') {
        const fields: Record<string, string> = {}
        const ft = fieldValues['field_fields_type']; if (ft) fields.type = ft
        const fc = fieldValues['field_fields_content']; if (fc) fields.content = fc
        const fd = fieldValues['field_fields_description']; if (fd) fields.description = fd
        body = { fields, link_to: node.id }
      } else {
        const fields: Record<string, string> = {}
        for (const [k, v] of Object.entries(fieldValues)) {
          if (k.startsWith('field_')) fields[k.slice(6)] = v
        }
        body = fields
      }

      const newNode = await apiPost<{ id: string }>(endpoint, body)

      if (addLinkType !== 'contexts') {
        const info = addLinkEdgeInfo
        if (info && !('manual' in info)) {
          await apiPost('/api/link', { from_id: node.id, edge: info.fwd, to_id: newNode.id })
          await apiPost('/api/link', { from_id: newNode.id, edge: info.rev, to_id: node.id })
        } else if (info && 'manual' in info) {
          await apiPost('/api/link', { from_id: node.id, edge: manualEdgeFwd, to_id: newNode.id })
        }
      }

      closeModal()
      setStatus('Created & linked', 'ok')
      onRefresh()
    } catch (e: unknown) {
      setStatus('Create & link failed: ' + (e as Error).message, 'err')
    }
  }

  function onOverlayClick(e: MouseEvent) {
    if ((e.target as HTMLElement).id === 'modal') closeModal()
  }
</script>

{#if $modalState.open}
  <div id="modal" class="modal-overlay" onclick={onOverlayClick} onkeydown={(e) => e.key === 'Escape' && closeModal()} role="dialog" aria-modal="true" tabindex="-1">
    <div class="modal">
      {#if $modalState.mode === 'create'}
        <h2>Create {$modalState.tab?.slice(0, -1)}</h2>
        <form>
          {#each SCHEMAS[$modalState.tab!] ?? [] as f}
            <label>
              <span>{f.label}{f.required ? ' *' : ''}</span>
              {#if f.type === 'textarea'}
                <textarea name={f.name} required={f.required} bind:value={fieldValues[f.name]}></textarea>
              {:else}
                <input type="text" name={f.name} required={f.required} bind:value={fieldValues[f.name]} />
              {/if}
            </label>
          {/each}
        </form>
        <div class="modal-btns">
          <button type="button" class="cancel" onclick={closeModal}>Cancel</button>
          <button type="button" class="submit" onclick={submitCreate}>Create</button>
        </div>

      {:else if $modalState.mode === 'edit' && $modalState.editingNode}
        <h2>Edit {$modalState.editingNode.node_type}: {$modalState.editingNode.label}</h2>
        <form>
          {#each EDIT_SCHEMAS[$modalState.editingNode.node_type] ?? [] as f}
            <label>
              <span>{f.label}</span>
              {#if f.type === 'textarea'}
                <textarea name={f.name} bind:value={fieldValues[f.name]}></textarea>
              {:else if f.type === 'select'}
                <select name={f.name} bind:value={fieldValues[f.name]}>
                  {#each f.options ?? [] as opt}
                    <option value={opt}>{opt}</option>
                  {/each}
                </select>
              {:else}
                <input type="text" name={f.name} bind:value={fieldValues[f.name]} />
              {/if}
            </label>
          {/each}
        </form>
        <div class="modal-btns">
          <button type="button" class="cancel" onclick={closeModal}>Cancel</button>
          <button type="button" class="submit" onclick={submitEdit}>Save</button>
        </div>

      {:else if $modalState.mode === 'add-link' && $modalState.editingNode}
        <h2>Add &amp; Link to: {$modalState.editingNode.label}</h2>
        <form>
          <label>
            <span>Node Type *</span>
            <select bind:value={addLinkType}>
              {#each Object.keys(CREATE_ENDPOINTS) as t}
                <option value={t}>{t}</option>
              {/each}
            </select>
          </label>
          {#each addLinkSchema as f}
            <label>
              <span>{f.label}{f.required ? ' *' : ''}</span>
              {#if f.type === 'textarea'}
                <textarea name={'field_' + f.name} required={f.required} bind:value={fieldValues['field_' + f.name]}></textarea>
              {:else}
                <input type="text" name={'field_' + f.name} required={f.required} bind:value={fieldValues['field_' + f.name]} />
              {/if}
            </label>
          {/each}
          {#if addLinkEdgeInfo && !('manual' in addLinkEdgeInfo)}
            <div class="edge-info">
              Edges: this → <em>{addLinkEdgeInfo.fwd}</em> → new &nbsp;|&nbsp; new → <em>{addLinkEdgeInfo.rev}</em> → this
            </div>
          {:else if addLinkEdgeInfo && 'manual' in addLinkEdgeInfo}
            <label>
              <span>Edge (this → new)</span>
              <select bind:value={manualEdgeFwd}>
                {#each ALL_EDGE_TYPES as e}
                  <option value={e}>{e}</option>
                {/each}
              </select>
            </label>
          {/if}
        </form>
        <div class="modal-btns">
          <button type="button" class="cancel" onclick={closeModal}>Cancel</button>
          <button type="button" class="submit" onclick={submitAddLink}>Create &amp; Link</button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.6);
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .modal {
    background: #1a1d27;
    border: 1px solid #3d4165;
    border-radius: 8px;
    padding: 20px;
    min-width: 360px;
    max-width: 500px;
    width: 90%;
  }
  .modal h2 { color: #a78bfa; margin-bottom: 14px; font-size: 16px; }
  .modal label { display: block; margin-bottom: 10px; }
  .modal label span { display: block; font-size: 12px; color: #94a3b8; margin-bottom: 4px; }
  .modal :global(input),
  .modal :global(textarea),
  .modal :global(select) {
    width: 100%;
    background: #252840;
    color: #e2e8f0;
    border: 1px solid #3d4165;
    padding: 6px 10px;
    border-radius: 4px;
    font-size: 13px;
    font-family: inherit;
  }
  .modal :global(textarea) { height: 80px; resize: vertical; }
  .modal-btns { display: flex; gap: 8px; justify-content: flex-end; margin-top: 16px; }
  .modal-btns button { padding: 6px 16px; border-radius: 4px; cursor: pointer; border: none; font-size: 13px; }
  .cancel { background: #252840; color: #94a3b8; }
  .submit { background: #5b45d6; color: #fff; }
  .submit:hover { background: #7c6df0; }
  .edge-info { margin-top: 8px; padding: 6px; background: #0f1117; border-radius: 4px; font-size: 11px; color: #94a3b8; }
</style>

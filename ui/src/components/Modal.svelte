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
  import * as Dialog from '$lib/components/ui/dialog'
  import { Button } from '$lib/components/ui/button'
  import { Input } from '$lib/components/ui/input'
  import { Textarea } from '$lib/components/ui/textarea'
  import { Label } from '$lib/components/ui/label'
  import * as Select from '$lib/components/ui/select'

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
</script>

<Dialog.Root open={$modalState.open} onOpenChange={(v) => { if (!v) closeModal() }}>
  <Dialog.Content class="bg-[#1a1d27] border-[#3d4165] min-w-[360px] max-w-[500px] w-[90%] sm:max-w-[500px]">
    {#if $modalState.mode === 'create'}
      <Dialog.Header>
        <Dialog.Title class="text-[#a78bfa]">Create {$modalState.tab?.slice(0, -1)}</Dialog.Title>
      </Dialog.Header>
      <div class="flex flex-col gap-2.5 max-h-[60vh] overflow-y-auto py-1 pr-1">
        {#each SCHEMAS[$modalState.tab!] ?? [] as f}
          <div>
            <Label class="text-[#94a3b8] text-xs mb-1 block">{f.label}{f.required ? ' *' : ''}</Label>
            {#if f.type === 'textarea'}
              <Textarea
                name={f.name}
                required={f.required}
                bind:value={fieldValues[f.name]}
                class="bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-[13px] min-h-20"
              />
            {:else}
              <Input
                type="text"
                name={f.name}
                required={f.required}
                bind:value={fieldValues[f.name]}
                class="bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-[13px] h-8"
              />
            {/if}
          </div>
        {/each}
      </div>
      <Dialog.Footer>
        <Button variant="ghost" class="text-[#94a3b8]" onclick={closeModal}>Cancel</Button>
        <Button onclick={submitCreate}>Create</Button>
      </Dialog.Footer>

    {:else if $modalState.mode === 'edit' && $modalState.editingNode}
      <Dialog.Header>
        <Dialog.Title class="text-[#a78bfa]">Edit {$modalState.editingNode.node_type}: {$modalState.editingNode.label}</Dialog.Title>
      </Dialog.Header>
      <div class="flex flex-col gap-2.5 max-h-[60vh] overflow-y-auto py-1 pr-1">
        {#each EDIT_SCHEMAS[$modalState.editingNode.node_type] ?? [] as f}
          <div>
            <Label class="text-[#94a3b8] text-xs mb-1 block">{f.label}</Label>
            {#if f.type === 'textarea'}
              <Textarea
                name={f.name}
                bind:value={fieldValues[f.name]}
                class="bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-[13px] min-h-20"
              />
            {:else if f.type === 'select'}
              <Select.Root
                type="single"
                value={fieldValues[f.name]}
                onValueChange={(v) => { fieldValues[f.name] = v }}
              >
                <Select.Trigger class="w-full h-8 bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-[13px]">
                  {fieldValues[f.name] || '— select —'}
                </Select.Trigger>
                <Select.Content>
                  {#each f.options ?? [] as opt}
                    <Select.Item value={opt} label={opt} />
                  {/each}
                </Select.Content>
              </Select.Root>
            {:else}
              <Input
                type="text"
                name={f.name}
                bind:value={fieldValues[f.name]}
                class="bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-[13px] h-8"
              />
            {/if}
          </div>
        {/each}
      </div>
      <Dialog.Footer>
        <Button variant="ghost" class="text-[#94a3b8]" onclick={closeModal}>Cancel</Button>
        <Button onclick={submitEdit}>Save</Button>
      </Dialog.Footer>

    {:else if $modalState.mode === 'add-link' && $modalState.editingNode}
      <Dialog.Header>
        <Dialog.Title class="text-[#a78bfa]">Add &amp; Link to: {$modalState.editingNode.label}</Dialog.Title>
      </Dialog.Header>
      <div class="flex flex-col gap-2.5 max-h-[60vh] overflow-y-auto py-1 pr-1">
        <div>
          <Label class="text-[#94a3b8] text-xs mb-1 block">Node Type *</Label>
          <Select.Root
            type="single"
            value={addLinkType}
            onValueChange={(v) => { addLinkType = v }}
          >
            <Select.Trigger class="w-full h-8 bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-[13px]">
              {addLinkType}
            </Select.Trigger>
            <Select.Content>
              {#each Object.keys(CREATE_ENDPOINTS) as t}
                <Select.Item value={t} label={t} />
              {/each}
            </Select.Content>
          </Select.Root>
        </div>
        {#each addLinkSchema as f}
          <div>
            <Label class="text-[#94a3b8] text-xs mb-1 block">{f.label}{f.required ? ' *' : ''}</Label>
            {#if f.type === 'textarea'}
              <Textarea
                name={'field_' + f.name}
                required={f.required}
                bind:value={fieldValues['field_' + f.name]}
                class="bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-[13px] min-h-20"
              />
            {:else}
              <Input
                type="text"
                name={'field_' + f.name}
                required={f.required}
                bind:value={fieldValues['field_' + f.name]}
                class="bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-[13px] h-8"
              />
            {/if}
          </div>
        {/each}
        {#if addLinkEdgeInfo && !('manual' in addLinkEdgeInfo)}
          <div class="mt-1 px-3 py-1.5 bg-[#0f1117] rounded text-[11px] text-[#94a3b8]">
            Edges: this → <em>{addLinkEdgeInfo.fwd}</em> → new &nbsp;|&nbsp; new → <em>{addLinkEdgeInfo.rev}</em> → this
          </div>
        {:else if addLinkEdgeInfo && 'manual' in addLinkEdgeInfo}
          <div>
            <Label class="text-[#94a3b8] text-xs mb-1 block">Edge (this → new)</Label>
            <Select.Root
              type="single"
              value={manualEdgeFwd}
              onValueChange={(v) => { manualEdgeFwd = v }}
            >
              <Select.Trigger class="w-full h-8 bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-[13px]">
                {manualEdgeFwd}
              </Select.Trigger>
              <Select.Content>
                {#each ALL_EDGE_TYPES as e}
                  <Select.Item value={e} label={e} />
                {/each}
              </Select.Content>
            </Select.Root>
          </div>
        {/if}
      </div>
      <Dialog.Footer>
        <Button variant="ghost" class="text-[#94a3b8]" onclick={closeModal}>Cancel</Button>
        <Button onclick={submitAddLink}>Create &amp; Link</Button>
      </Dialog.Footer>
    {/if}
  </Dialog.Content>
</Dialog.Root>

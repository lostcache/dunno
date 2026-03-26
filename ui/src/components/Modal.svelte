<script lang="ts">
  import { get } from 'svelte/store'
  import { modalState, closeModal } from '../stores/modalStore'
  import { projectId } from '../stores/appStore'
  import { editingNode } from '../stores/graphStore'
  import { api, apiPost, apiPatch } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import {
    SCHEMAS, EDIT_SCHEMAS, EDIT_ENDPOINTS, CREATE_ENDPOINTS,
  } from '../lib/constants'
  import type { SchemaField } from '../lib/types'
  import * as Dialog from '$lib/components/ui/dialog'
  import { Button } from '$lib/components/ui/button'
  import { Input } from '$lib/components/ui/input'
  import { Label } from '$lib/components/ui/label'
  import MarkdownEditor from './MarkdownEditor.svelte'
  import * as Select from '$lib/components/ui/select'

  let { onRefresh }: { onRefresh: () => void } = $props()

  // Form field values — keyed by field name
  let fieldValues = $state<Record<string, string>>({})
  let taskOptions = $state<{ id: string; name: string }[]>([])

  async function loadTaskOptions() {
    const pid = get(projectId)
    if (!pid) { taskOptions = []; return }
    try {
      const tasks = await api<{ id: string; name: string }[]>(`/api/projects/${pid}/tasks`)
      taskOptions = tasks
    } catch {
      taskOptions = []
    }
  }

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
      const needsTasks = schema.some(f => f.fill === 'taskId')
      if (needsTasks) loadTaskOptions()
      for (const f of schema) {
        newValues[f.name] = f.fill === 'projectId' ? pid : ''
      }
      fieldValues = newValues
    }
  }

  $effect.pre(() => {
    if ($modalState.open) initFields()
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
      body = Object.fromEntries(Object.entries(fieldValues).filter(([, v]) => v !== ''))
      const schema = SCHEMAS[tab] ?? []
      for (const f of schema) {
        if (f.required && !(f.name in (body as Record<string, unknown>))) {
          setStatus(`${f.label} is required`, 'err')
          return
        }
      }
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
            {#if f.fill === 'taskId'}
              <Select.Root
                type="single"
                value={fieldValues[f.name]}
                onValueChange={(v) => { fieldValues[f.name] = v }}
              >
                <Select.Trigger class="w-full h-8 bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-[13px]">
                  {taskOptions.find(t => t.id === fieldValues[f.name])?.name || '— none —'}
                </Select.Trigger>
                <Select.Content>
                  {#each taskOptions as t}
                    <Select.Item value={t.id} label={t.name} />
                  {/each}
                </Select.Content>
              </Select.Root>
            {:else if f.type === 'textarea'}
              <MarkdownEditor bind:value={fieldValues[f.name]} />
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
              <MarkdownEditor bind:value={fieldValues[f.name]} />
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

    {/if}
  </Dialog.Content>
</Dialog.Root>

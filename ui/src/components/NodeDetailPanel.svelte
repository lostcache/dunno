<script lang="ts">
  import { get } from 'svelte/store'
  import { editingNode, cyInstance } from '../stores/graphStore'
  import { openEdit } from '../stores/modalStore'
  import { apiDel, apiPatch } from '../lib/api'
  import { setStatus } from '../stores/statusStore'
  import { EDIT_SCHEMAS, EDIT_ENDPOINTS } from '../lib/constants'
  import { Button } from '$lib/components/ui/button'
  import { Badge } from '$lib/components/ui/badge'
  import { Separator } from '$lib/components/ui/separator'
  import { ScrollArea } from '$lib/components/ui/scroll-area'
  import { Card, CardHeader, CardTitle, CardContent } from '$lib/components/ui/card'
  import { Input } from '$lib/components/ui/input'
  import { Label } from '$lib/components/ui/label'
  import * as Select from '$lib/components/ui/select'
  import MarkdownEditor from './MarkdownEditor.svelte'

  let { onRefresh }: { onRefresh: () => Promise<void> } = $props()

  let confirmingDelete = $state(false)
  let fieldValues = $state<Record<string, string>>({})
  let panelWidth = $state(340)

  function startResize(e: MouseEvent) {
    e.preventDefault()
    const startX = e.clientX
    const startWidth = panelWidth

    function onMove(ev: MouseEvent) {
      panelWidth = Math.min(640, Math.max(200, startWidth - (ev.clientX - startX)))
    }
    function onUp() {
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
  }

  // Must run before DOM update so bind:value={fieldValues[f.name]} is never undefined
  // when MarkdownEditor mounts (Svelte 5 throws props_invalid_value on bind:value={undefined})
  $effect.pre(() => {
    const node = $editingNode
    if (!node) return
    const schema = EDIT_SCHEMAS[node.node_type] ?? []
    const newValues: Record<string, string> = {}
    for (const f of schema) {
      newValues[f.name] = String((node as Record<string, unknown>)[f.name] ?? '')
    }
    fieldValues = newValues
    confirmingDelete = false
  })

  // Redirect to modal on small screens — runs after DOM update to avoid SSR issues with window
  $effect(() => {
    const node = $editingNode
    if (!node) return
    if (window.innerWidth < 1024) {
      openEdit(node)
      editingNode.set(null)
    }
  })

  function toTitleCase(key: string): string {
    return key.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
  }

  function getExtraFields(node: Record<string, unknown>): [string, unknown][] {
    const schemaKeys = new Set((EDIT_SCHEMAS[node.node_type as string] ?? []).map(f => f.name))
    return Object.entries(node).filter(
      ([k, v]) =>
        k !== 'id' && k !== 'label' && k !== 'node_type' &&
        v !== node.label &&
        !schemaKeys.has(k)
    )
  }

  async function saveNode() {
    const node = $editingNode
    if (!node) return
    const endpoint = EDIT_ENDPOINTS[node.node_type]
    if (!endpoint) { setStatus('No endpoint for ' + node.node_type, 'err'); return }

    const body = node.node_type === 'context'
      ? { fields: { ...fieldValues } }
      : { ...fieldValues }

    const savedNodeId = node.id
    try {
      await apiPatch(`${endpoint}/${encodeURIComponent(node.id)}`, body)
      setStatus('Saved', 'ok')
      await onRefresh()
      get(cyInstance)?.getElementById(savedNodeId).select()
    } catch (e: unknown) {
      setStatus('Save failed: ' + (e as Error).message, 'err')
    }
  }

  async function deleteNode() {
    const data = $editingNode
    if (!data) return
    const base = EDIT_ENDPOINTS[data.node_type]
    if (!base) { setStatus('No delete endpoint for ' + data.node_type, 'err'); return }
    try {
      await apiDel(`${base}/${encodeURIComponent(data.id)}`)
      editingNode.set(null)
      confirmingDelete = false
      setStatus('Deleted ' + data.label, 'ok')
      onRefresh()
    } catch (e: unknown) {
      setStatus('Delete failed: ' + (e as Error).message, 'err')
    }
  }
</script>

{#if $editingNode}
  <div class="max-lg:hidden relative flex flex-col border-l border-[#2d3148] bg-[#14172a] overflow-hidden shrink-0" style="width: {panelWidth}px">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      role="separator"
      aria-label="Resize sidebar"
      class="absolute left-0 top-0 h-full w-1 cursor-col-resize hover:bg-[#7c6df0]/40 z-10"
      onmousedown={startResize}
    ></div>
    <Card class="rounded-none border-0 bg-transparent flex flex-col h-full">
      <CardHeader class="pb-2 pt-3 px-3 gap-1.5">
        <div class="flex items-start justify-between gap-2">
          <div class="flex flex-col gap-1 min-w-0 overflow-hidden">
            <Badge variant="secondary" class="w-fit text-[10px]">{$editingNode.node_type}</Badge>
            <CardTitle class="text-sm text-[#a78bfa] leading-tight break-words">{$editingNode.label}</CardTitle>
          </div>
          <div class="flex flex-col gap-1 shrink-0">
            <Button size="sm" class="h-6 px-2 text-xs" onclick={saveNode}>Save</Button>
            {#if confirmingDelete}
              <div class="flex flex-col gap-1">
                <Button size="sm" variant="destructive" class="h-6 px-2 text-xs" onclick={deleteNode}>Confirm</Button>
                <Button size="sm" variant="ghost" class="h-6 px-2 text-xs" onclick={() => confirmingDelete = false}>Cancel</Button>
              </div>
            {:else}
              <Button size="sm" variant="destructive" class="h-6 px-2 text-xs" onclick={() => confirmingDelete = true}>Delete</Button>
            {/if}
          </div>
        </div>
      </CardHeader>

      <Separator class="bg-[#2d3148]" />

      <CardContent class="flex-1 p-0 overflow-hidden">
        <ScrollArea class="h-full px-3 py-2">
          <div class="flex flex-col gap-3">
            {#each EDIT_SCHEMAS[$editingNode.node_type] ?? [] as f}
              <div class="flex flex-col gap-1">
                <Label class="text-[10px] text-[#64748b] uppercase tracking-wide">{f.label}</Label>
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

            {#if getExtraFields($editingNode).length > 0}
              {#if (EDIT_SCHEMAS[$editingNode.node_type] ?? []).length > 0}
                <Separator class="bg-[#2d3148]" />
              {/if}
              {#each getExtraFields($editingNode) as [key, value]}
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
            {/if}
          </div>

          <div class="mt-3 pt-2 border-t border-[#2d3148]">
            <span
              class="text-[9px] text-[#475569] font-mono break-all cursor-pointer hover:text-[#64748b]"
              onclick={() => navigator.clipboard.writeText($editingNode.id).then(() => setStatus('ID copied', 'ok'))}
            >{$editingNode.id}</span>
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  </div>
{/if}

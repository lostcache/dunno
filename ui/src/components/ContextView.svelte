<script lang="ts">
  import { projectId } from "../stores/appStore";
  import { api } from "../lib/api";
  import { setStatus } from "../stores/statusStore";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";

  interface Task {
    id: string;
    name?: string;
    title?: string;
  }

  let ctxType = $state<"task" | "file" | "epic">("task");
  let ctxId = $state("");
  let ctxFull = $state(false);
  let ctxOutput = $state("Select type and enter an ID, then click Fetch.");
  let tasks = $state<Task[]>([]);

  async function loadTasks() {
    const pid = $projectId;
    if (!pid) {
      tasks = [];
      return;
    }
    try {
      tasks = await api<Task[]>(`/api/projects/${pid}/tasks`);
    } catch {
      tasks = [];
    }
  }

  $effect(() => {
    if (ctxType === "task") {
      $projectId; // track
      loadTasks();
    }
  });

  // Set ctxId to first task when tasks load
  $effect(() => {
    if (ctxType === "task" && tasks.length > 0 && !ctxId) {
      ctxId = tasks[0].id;
    }
  });

  let selectedTaskName = $derived(
    ctxId
      ? (tasks.find((t) => t.id === ctxId)?.title ??
          tasks.find((t) => t.id === ctxId)?.name ??
          ctxId)
      : "— select task —",
  );

  async function fetchCtx() {
    if (!ctxId) {
      setStatus("Enter an ID", "err");
      return;
    }
    try {
      const data = await api(`/api/ctx/${ctxType}/${encodeURIComponent(ctxId)}?full=${ctxFull}`);
      ctxOutput = JSON.stringify(data, null, 2);
      setStatus("Context fetched", "ok");
    } catch (e: unknown) {
      setStatus("Fetch failed: " + (e as Error).message, "err");
    }
  }
</script>

<div class="flex-1 p-5 flex flex-col gap-3 min-h-0 overflow-hidden">
  <div class="flex gap-2 items-center">
    <Select.Root
      type="single"
      value={ctxType}
      onValueChange={(v) => {
        ctxType = v as "task" | "file" | "epic";
        ctxId = "";
      }}
    >
      <Select.Trigger
        class="h-8 w-28 bg-[#252840] border-[#3d4165] text-[#e2e8f0] hover:bg-[#3d4165] text-xs"
      >
        {ctxType.charAt(0).toUpperCase() + ctxType.slice(1)}
      </Select.Trigger>
      <Select.Content>
        <Select.Item value="task" label="Task" />
        <Select.Item value="file" label="File" />
        <Select.Item value="epic" label="Epic" />
      </Select.Content>
    </Select.Root>

    <div class="flex-1 flex">
      {#if ctxType === "task" && tasks.length > 0}
        <Select.Root
          type="single"
          value={ctxId}
          onValueChange={(v) => {
            ctxId = v;
          }}
        >
          <Select.Trigger
            class="flex-1 h-8 bg-[#252840] border-[#3d4165] text-[#e2e8f0] hover:bg-[#3d4165] text-xs"
          >
            {selectedTaskName}
          </Select.Trigger>
          <Select.Content>
            {#each tasks as t}
              <Select.Item value={t.id} label={t.title || t.name || t.id} />
            {/each}
          </Select.Content>
        </Select.Root>
      {:else}
        <Input
          bind:value={ctxId}
          placeholder={ctxType === "task" ? "No tasks found" : "record id e.g. task:abc123"}
          class="flex-1 h-8 bg-[#252840] border-[#3d4165] text-[#e2e8f0] text-xs placeholder:text-[#64748b]"
        />
      {/if}
    </div>

    <Label class="flex items-center gap-1.5 text-[#94a3b8] text-xs cursor-pointer font-normal">
      <Checkbox bind:checked={ctxFull} class="size-3.5" />
      Full
    </Label>

    <Button size="sm" class="h-8 px-3.5 text-xs" onclick={fetchCtx}>Fetch</Button>
  </div>

  <div class="flex-1 overflow-y-auto min-h-0">
    <pre
      class="bg-[#14172a] border border-[#2d3148] p-3 rounded-md whitespace-pre-wrap break-all text-[#94a3b8] text-xs">{ctxOutput}</pre>
  </div>
</div>

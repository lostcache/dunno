<script lang="ts">
  import { statusState } from "../stores/statusStore";
</script>

<div
  class="flex-shrink-0 border-t border-[hsl(var(--border))] bg-[#14172a] px-4 py-1 min-h-6 text-xs"
  class:text-green-400={$statusState.cls === "ok"}
  class:text-red-400={$statusState.cls === "err"}
  class:text-[#64748b]={$statusState.cls !== "ok" && $statusState.cls !== "err"}
>
  {$statusState.message}
</div>

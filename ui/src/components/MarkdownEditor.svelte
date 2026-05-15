<script lang="ts">
  import { onDestroy } from "svelte";
  import { Editor } from "@milkdown/kit/core";
  import { commonmark } from "@milkdown/kit/preset/commonmark";
  import { listener, listenerCtx } from "@milkdown/kit/plugin/listener";
  import { rootCtx, defaultValueCtx } from "@milkdown/kit/core";
  import { replaceAll } from "@milkdown/kit/utils";

  import "@milkdown/kit/prose/view/style/prosemirror.css";

  let { value = $bindable("") }: { value: string } = $props();

  let editor: Editor | null = null;
  let editorReady = $state(false);
  let prevValue = value;

  function milkdown(node: HTMLElement) {
    const initialValue = value;
    editor = Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, node);
        ctx.set(defaultValueCtx, initialValue ?? "");
        ctx.get(listenerCtx).markdownUpdated((_ctx, md) => {
          prevValue = md;
          value = md;
        });
      })
      .use(commonmark)
      .use(listener);

    editor
      .create()
      .then(() => {
        editorReady = true;
        // If value was updated before the editor finished initializing, apply it now
        if (value !== initialValue) {
          editor?.action(replaceAll(value ?? ""));
          prevValue = value;
        }
      })
      .catch(console.error);

    return {
      destroy() {
        editorReady = false;
        editor?.destroy();
        editor = null;
      },
    };
  }

  $effect(() => {
    if (editorReady && value !== prevValue) {
      editor?.action(replaceAll(value ?? ""));
      prevValue = value;
    }
  });

  onDestroy(() => {
    editor?.destroy();
    editor = null;
  });
</script>

<div use:milkdown class="milkdown-editor"></div>

<style>
  .milkdown-editor {
    --milkdown-color-bg: #252840;
    --milkdown-color-fg: #e2e8f0;
    --milkdown-color-border: #3d4165;
    --milkdown-color-highlight: #a78bfa;
  }

  .milkdown-editor :global(.milkdown) {
    background: #252840;
    color: #e2e8f0;
    border: 1px solid #3d4165;
    border-radius: 6px;
    min-height: 80px;
    padding: 8px 10px;
    font-size: 13px;
    line-height: 1.6;
    outline: none;
  }

  .milkdown-editor :global(.milkdown:focus-within) {
    border-color: #a78bfa;
  }

  .milkdown-editor :global(.ProseMirror) {
    outline: none;
    min-height: 60px;
  }

  .milkdown-editor :global(.ProseMirror p) {
    margin: 0 0 4px;
  }

  .milkdown-editor :global(.ProseMirror h1),
  .milkdown-editor :global(.ProseMirror h2),
  .milkdown-editor :global(.ProseMirror h3) {
    color: #a78bfa;
    margin: 6px 0 4px;
  }

  .milkdown-editor :global(.ProseMirror code) {
    background: #1a1d27;
    color: #a78bfa;
    padding: 1px 4px;
    border-radius: 3px;
    font-size: 12px;
  }

  .milkdown-editor :global(.ProseMirror pre) {
    background: #1a1d27;
    padding: 8px;
    border-radius: 4px;
    overflow-x: auto;
  }

  .milkdown-editor :global(.ProseMirror blockquote) {
    border-left: 3px solid #3d4165;
    padding-left: 8px;
    color: #94a3b8;
    margin: 4px 0;
  }

  .milkdown-editor :global(.ProseMirror a) {
    color: #a78bfa;
  }

  .milkdown-editor :global(.ProseMirror ul),
  .milkdown-editor :global(.ProseMirror ol) {
    padding-left: 20px;
    margin: 4px 0;
  }

  .milkdown-editor :global(.ProseMirror strong) {
    color: #f1f5f9;
  }
</style>

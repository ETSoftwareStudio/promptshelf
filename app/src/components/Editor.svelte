<!-- The prompt file as Markdown with its frontmatter. Save writes it only when
     it still parses as a prompt. -->
<script lang="ts">
  import { modKey } from "../lib/hotkey";

  let {
    text = $bindable(),
    path,
    error = null,
    saving = false,
    onsave,
    oncancel,
  }: {
    text: string;
    path: string;
    error?: string | null;
    saving?: boolean;
    onsave: () => void;
    oncancel: () => void;
  } = $props();

  let area = $state<HTMLTextAreaElement>();
  $effect(() => {
    area?.focus();
  });

  function onkeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "s") {
      e.preventDefault();
      onsave();
    } else if (e.key === "Escape") {
      e.preventDefault();
      oncancel();
    } else if (e.key === "Tab" && !e.shiftKey) {
      // Tab types two spaces, as YAML wants.
      e.preventDefault();
      const el = e.currentTarget as HTMLTextAreaElement;
      el.setRangeText("  ", el.selectionStart, el.selectionEnd, "end");
      text = el.value;
    }
  }
</script>

<div class="editor">
  {#if error}<div class="banner" role="alert">{error}</div>{/if}
  <textarea
    bind:this={area}
    bind:value={text}
    class="src"
    spellcheck="false"
    aria-label="Prompt file"
    {onkeydown}
  ></textarea>
  <div class="foot">
    <span class="path mono" title={path}>{path}</span>
    <button class="btn" onclick={oncancel}>Cancel</button>
    <button class="btn primary" onclick={onsave} disabled={saving}
      >{saving ? "Saving…" : "Save"} <span class="k">{modKey("S")}</span></button
    >
  </div>
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    gap: 10px;
  }
  .src {
    flex: 1;
    min-height: 160px;
    resize: none;
    padding: 12px 14px;
    border: 0;
    border-radius: 9px;
    background: var(--group);
    box-shadow: inset 0 0 0 0.5px var(--edge);
    font: 12.5px/1.6 var(--font-mono);
    color: var(--ink);
    tab-size: 2;
    user-select: text;
    -webkit-user-select: text;
  }
  .src:focus {
    outline: 2px solid var(--focus);
    outline-offset: 0;
  }
  .foot {
    display: flex;
    align-items: center;
    gap: 7px;
    padding-bottom: 12px;
  }
  .path {
    margin-right: auto;
    color: var(--ink3);
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .k {
    opacity: 0.75;
    font-weight: 400;
  }
</style>

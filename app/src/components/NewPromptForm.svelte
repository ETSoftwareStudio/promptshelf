<!-- The right column for a new prompt: a title, a kind, a library and the
     text. Nothing is written until Save. -->
<script lang="ts">
  import { DRAFT_KINDS, type Draft } from "../lib/draft";
  import { libraryLabel } from "../lib/format";
  import { modKey } from "../lib/hotkey";
  import { typeTab } from "../lib/keys";
  import type { LibraryInfo } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    draft = $bindable(),
    libraries,
    saving = false,
    error = null,
    paused = false,
    onsave,
    ondiscard,
  }: {
    draft: Draft;
    libraries: LibraryInfo[];
    saving?: boolean;
    error?: string | null;
    /** True while a sheet is open over the form: ⌘S waits. */
    paused?: boolean;
    onsave: () => void;
    ondiscard: () => void;
  } = $props();

  let titleEl = $state<HTMLInputElement>();
  let bodyEl = $state<HTMLTextAreaElement>();

  /** Puts the cursor in the title, for ⌘N while the form is open. */
  export function focus() {
    titleEl?.focus();
  }

  $effect(() => {
    titleEl?.focus();
  });

  const team = $derived(libraries.find((l) => l.name === draft.library)?.writes === "pull-request");

  function onwindowkey(e: KeyboardEvent) {
    if (paused || !(e.metaKey || e.ctrlKey) || e.altKey || e.shiftKey) return;
    if (e.key === "s") {
      e.preventDefault();
      if (!saving) onsave();
    }
  }

  function onformkey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      // The confirm that may open listens for Esc on the window: this press
      // must not reach it and close it at once.
      e.preventDefault();
      e.stopPropagation();
      ondiscard();
    }
  }

  function ontitlekey(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.metaKey && !e.ctrlKey) {
      e.preventDefault();
      bodyEl?.focus();
    }
  }

  function onbodykey(e: KeyboardEvent) {
    const next = typeTab(e);
    if (next !== null) draft.body = next;
  }
</script>

<svelte:window onkeydown={onwindowkey} />

<section class="detail" aria-label="New prompt">
  <header class="dh" data-tauri-drag-region>
    <span class="what" data-tauri-drag-region>New prompt · not saved yet</span>
    <button class="btn" onclick={ondiscard}>Discard</button>
    <button class="btn primary" onclick={onsave} disabled={saving}>
      {saving ? "Saving…" : "Save"} <span class="k">{modKey("S")}</span>
    </button>
  </header>

  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="body" role="group" onkeydown={onformkey}>
    {#if error}
      <div class="banner" role="alert"><Icon name="warn" small /><span class="grow">{error}</span></div>
    {/if}
    <input
      bind:this={titleEl}
      bind:value={draft.title}
      class="input title"
      placeholder="Untitled"
      aria-label="Title"
      spellcheck="false"
      onkeydown={ontitlekey}
    />
    <div class="meta">
      <label>
        <span>Kind</span>
        <select bind:value={draft.kind} class="input">
          {#each DRAFT_KINDS as k (k.kind)}
            <option value={k.kind}>{k.label}</option>
          {/each}
        </select>
      </label>
      <label>
        <span>Library</span>
        <select bind:value={draft.library} class="input">
          {#each libraries as lib (lib.name)}
            <option value={lib.name}>{libraryLabel(lib.name)}</option>
          {/each}
        </select>
      </label>
    </div>
    <textarea
      bind:this={bodyEl}
      bind:value={draft.body}
      class="input text"
      placeholder={"Write the prompt here. Use {{name}} for a blank."}
      aria-label="Prompt text"
      onkeydown={onbodykey}
    ></textarea>
    <p class="note">
      {#if team}
        {libraryLabel(draft.library)} is a team library. Save makes a proposal on this computer.
      {:else}
        Nothing is written until you save.
      {/if}
    </p>
  </div>
</section>

<style>
  .detail {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    background: var(--ground);
  }
  .dh {
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 7px;
    padding: 0 12px 0 22px;
    flex: none;
  }
  .what {
    margin-right: auto;
    font-size: 12px;
    color: var(--ink3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .k {
    opacity: 0.75;
    font-weight: 400;
  }
  .body {
    padding: 4px 22px 0;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .banner {
    margin-bottom: 10px;
  }
  .title {
    font-size: 19px;
    font-weight: 650;
    letter-spacing: -0.01em;
    padding: 5px 9px;
    border-radius: 8px;
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    margin: 10px 0 14px;
    font-size: 12px;
    color: var(--ink2);
  }
  .meta label {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .meta select {
    width: auto;
    padding-top: 3px;
    padding-bottom: 3px;
  }
  .text {
    flex: 1;
    min-height: 160px;
    resize: none;
    padding: 12px 14px;
    border-radius: 9px;
    font-family: var(--font-prompt);
    font-size: 15.5px;
    line-height: 1.7;
    tab-size: 2;
  }
  .note {
    margin: 0;
    padding: 9px 0 12px;
    font-size: 11.5px;
    color: var(--ink3);
  }
</style>

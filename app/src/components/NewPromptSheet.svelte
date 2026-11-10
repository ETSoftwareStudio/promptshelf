<!-- A new prompt: a title, a kind and a library. The file opens in the editor. -->
<script lang="ts">
  import { libraryLabel } from "../lib/format";
  import type { Kind, LibraryInfo } from "../lib/types";

  let {
    libraries,
    personal,
    oncreate,
    oncancel,
  }: {
    libraries: LibraryInfo[];
    personal: string | null | undefined;
    oncreate: (title: string, kind: Kind, library: string) => void;
    oncancel: () => void;
  } = $props();

  let title = $state("");
  let kind = $state<Kind>("template");
  // svelte-ignore state_referenced_locally
  let library = $state(personal ?? libraries[0]?.name ?? "");
  let input = $state<HTMLInputElement>();
  $effect(() => {
    input?.focus();
  });

  function submit(e: SubmitEvent) {
    e.preventDefault();
    oncreate(title.trim() || "Untitled", kind, library);
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && oncancel()} />

<div class="dim" role="presentation" onclick={oncancel}></div>
<form class="sheet" aria-label="New prompt" onsubmit={submit}>
  <h3>New prompt</h3>
  <div class="sub">A Markdown file. It opens in the editor.</div>
  <label class="f">
    <span>Title</span>
    <input bind:this={input} bind:value={title} class="input" placeholder="Strict PR review" />
  </label>
  <div class="row2">
    <label class="f">
      <span>Kind</span>
      <select bind:value={kind} class="input">
        <option value="template">Template</option>
        <option value="snippet">Snippet</option>
        <option value="context">Context</option>
        <option value="draft">Draft (Inbox)</option>
      </select>
    </label>
    <label class="f">
      <span>Library</span>
      <select bind:value={library} class="input">
        {#each libraries as lib (lib.name)}
          <option value={lib.name}>{libraryLabel(lib.name)}</option>
        {/each}
      </select>
    </label>
  </div>
  <div class="sfoot">
    <button type="button" class="btn" onclick={oncancel}>Cancel</button>
    <button type="submit" class="btn primary">Create</button>
  </div>
</form>

<style>
  .f {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin-top: 10px;
    font-size: 11.5px;
    color: var(--ink2);
    flex: 1;
  }
  .row2 {
    display: flex;
    gap: 10px;
  }
</style>

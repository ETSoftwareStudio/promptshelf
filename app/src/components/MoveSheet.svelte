<!-- Keep a draft: move it from the Inbox into a library as a prompt. A team
     library gets it as a proposal. -->
<script lang="ts">
  import { libraryLabel } from "../lib/format";
  import type { Kind, LibraryInfo } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    title,
    libraries,
    personal,
    error = null,
    busy = false,
    onmove,
    oncancel,
  }: {
    title: string;
    libraries: LibraryInfo[];
    personal: string | null | undefined;
    error?: string | null;
    busy?: boolean;
    onmove: (library: string, kind: Kind) => void;
    oncancel: () => void;
  } = $props();

  // svelte-ignore state_referenced_locally
  let library = $state(personal ?? libraries[0]?.name ?? "");
  let kind = $state<Kind>("template");
  const team = $derived(libraries.find((l) => l.name === library)?.writes === "pull-request");
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && oncancel()} />

<div class="dim" role="presentation" onclick={() => !busy && oncancel()}></div>
<form class="sheet" aria-label="Keep {title}" onsubmit={(e) => (e.preventDefault(), onmove(library, kind))}>
  <h3>Keep “{title}”</h3>
  <div class="sub">Moves the draft out of the Inbox and into a library.</div>
  <div class="field-row">
    <label class="field">
      <span>Library</span>
      <select class="input" bind:value={library}>
        {#each libraries as lib (lib.name)}
          <option value={lib.name}>{libraryLabel(lib.name)}{lib.writes === "pull-request" ? " (team)" : ""}</option>
        {/each}
      </select>
    </label>
    <label class="field">
      <span>Kind</span>
      <select class="input" bind:value={kind}>
        <option value="template">Template</option>
        <option value="snippet">Snippet</option>
        <option value="context">Context</option>
      </select>
    </label>
  </div>
  {#if team}<p class="fine">{libraryLabel(library)} is a team library: the prompt becomes a proposal. Send it when it is ready.</p>{/if}
  {#if error}<div class="banner err" role="alert"><Icon name="warn" small /><span class="grow">{error}</span></div>{/if}
  <div class="sfoot">
    <button type="button" class="btn" onclick={oncancel} disabled={busy}>Cancel</button>
    <button type="submit" class="btn primary" disabled={busy}>{busy ? "Moving…" : "Move to Library"}</button>
  </div>
</form>

<style>
  .banner {
    margin: 10px 0 0;
  }
</style>

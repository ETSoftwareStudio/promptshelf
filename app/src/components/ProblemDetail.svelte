<!-- A file that could not be read as a prompt: what is wrong, and the file to
     fix it. Save writes it only when it reads as a prompt. -->
<script lang="ts">
  import { api, errorText } from "../lib/api";
  import { libraryLabel } from "../lib/format";
  import type { Diagnostic } from "../lib/types";
  import Editor from "./Editor.svelte";
  import Icon from "./Icon.svelte";

  let { problem, onfixed }: { problem: Diagnostic; onfixed: () => void } = $props();

  let text = $state<string | null>(null);
  let error = $state<string | null>(null);
  let saving = $state(false);
  const isFile = $derived(problem.path.endsWith(".md"));

  $effect(() => {
    const path = problem.path;
    text = null;
    error = null;
    if (path.endsWith(".md")) {
      api.readFile(path).then(
        (t) => (text = t),
        (e) => (error = errorText(e)),
      );
    }
  });

  async function save() {
    if (text === null) return;
    saving = true;
    try {
      await api.saveRawFile(problem.path, text);
      onfixed();
    } catch (e) {
      error = errorText(e);
    } finally {
      saving = false;
    }
  }
</script>

<section class="detail">
  <header class="dh" data-tauri-drag-region></header>
  <div class="body">
    <h2>{problem.file}</h2>
    <div class="meta">{problem.library ? libraryLabel(problem.library) : "Library"} · could not be read</div>
    <div class="banner" role="alert">
      <Icon name="warn" small />
      <div class="grow">
        <p>{problem.help}</p>
        <p class="mono raw">{problem.message}</p>
      </div>
    </div>
    {#if isFile && text !== null}
      <Editor bind:text path={problem.path} {error} {saving} onsave={() => void save()} />
    {:else if error}
      <div class="banner err"><Icon name="warn" small /><span class="grow">{error}</span></div>
    {/if}
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
    flex: none;
  }
  .body {
    padding: 4px 22px 0;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  h2 {
    margin: 0;
    font-size: 19px;
    font-weight: 650;
    overflow-wrap: anywhere;
  }
  .meta {
    font-size: 12px;
    color: var(--ink2);
    margin: 3px 0 14px;
  }
  .raw {
    color: var(--ink2);
    font-size: 11px;
  }
</style>

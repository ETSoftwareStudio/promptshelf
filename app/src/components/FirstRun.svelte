<!-- First run: no config yet. Choose where the Personal library lives: a new
     folder, a folder you have, or a clone of a git repo. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorText, openLink } from "../lib/api";
  import type { Environment } from "../lib/types";
  import Icon from "./Icon.svelte";

  let { ondone }: { ondone: (path: string) => void } = $props();

  const DEFAULT = "~/Prompts/personal";
  let path = $state(DEFAULT);
  let cloning = $state(false);
  let url = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  let env = $state<Environment | null>(null);
  const noGit = $derived(env !== null && !env.git.version);

  onMount(() => {
    void api.environment().then((e) => (env = e));
  });

  async function choose() {
    const picked = await api.pickFolder("Choose a folder for your prompts", path);
    if (picked) path = picked;
  }

  async function start(e?: SubmitEvent) {
    e?.preventDefault();
    if (cloning && !url.trim()) {
      error = "Paste the address of the git repo to clone.";
      return;
    }
    busy = true;
    error = null;
    try {
      ondone(await api.initPersonal(path, cloning ? url : null));
    } catch (err) {
      error = errorText(err);
    } finally {
      busy = false;
    }
  }
</script>

<main class="first">
  <aside class="side-empty" data-tauri-drag-region></aside>
  <div class="empty" data-tauri-drag-region>
    <Icon name="books" class="glyph" />
    <h4>No prompts yet</h4>
    <p>
      Prompts live in libraries: folders of Markdown files, usually git repos. Start with your
      Personal library. You can add your team's library later in Settings.
    </p>
    <form class="grp setup" onsubmit={start}>
      {#if noGit}
        <div class="banner" role="alert">
          <Icon name="warn" small />
          <div class="grow">
            <p>Git is not installed. Your prompts work without it; sync and teams need it.</p>
            <div class="acts">
              <button type="button" class="btn sm" onclick={() => openLink("https://git-scm.com/downloads")}>
                Get Git
              </button>
            </div>
          </div>
        </div>
      {/if}
      {#if cloning}
        <label class="field">
          <span>Git repo to clone</span>
          <input class="input mono" bind:value={url} placeholder="git@github.com:you/prompts.git" spellcheck="false" />
        </label>
      {/if}
      <div class="field">
        <span id="where">{cloning ? "Clone into" : "Folder"}</span>
        <div class="field-row">
          <input class="input mono" bind:value={path} aria-labelledby="where" spellcheck="false" />
          <button type="button" class="btn sm" onclick={() => void choose()}>Choose…</button>
        </div>
      </div>
      <p class="fine">
        {cloning
          ? "The folder must be empty. PromptShelf clones the repo into it."
          : "A new folder gets the standard layout. A folder that has prompts keeps its files."}
      </p>
      {#if error}<div class="banner err" role="alert"><Icon name="warn" small /><span class="grow">{error}</span></div>{/if}
      <div class="sfoot">
        <button
          type="button"
          class="btn grow"
          disabled={noGit && !cloning}
          onclick={() => ((cloning = !cloning), (error = null))}
        >
          {cloning ? "Use a Folder Instead" : "Clone from Git…"}
        </button>
        <button type="submit" class="btn primary" disabled={busy || (cloning && noGit)}>
          {busy ? "Setting up…" : cloning ? "Clone and Start" : "Create Personal Library"}
        </button>
      </div>
    </form>
  </div>
</main>

<style>
  .first {
    display: grid;
    grid-template-columns: minmax(176px, 200px) 1fr;
    height: 100%;
  }
  .first .empty {
    background: var(--ground);
  }
  .side-empty {
    margin: 8px 0 8px 8px;
    border-radius: 14px;
    background: var(--side);
    box-shadow: inset 0 0 0 0.5px var(--edge);
  }
  :global(html.vibrant) .side-empty {
    background: color-mix(in oklch, var(--side) 55%, transparent);
  }
  .setup {
    width: min(420px, 100%);
    padding: 4px 14px 12px;
    text-align: left;
  }
  .setup .banner {
    margin: 10px 0 0;
  }
</style>

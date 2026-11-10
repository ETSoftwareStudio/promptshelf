<!-- Add a library: a git repo to clone, or a folder on this computer. -->
<script lang="ts">
  import { dim, sheet } from "../lib/motion";
  import { api, errorText } from "../lib/api";
  import type { Writes } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    hasGit = true,
    onadded,
    oncancel,
  }: { hasGit?: boolean; onadded: (name: string) => void; oncancel: () => void } = $props();

  // svelte-ignore state_referenced_locally
  let mode = $state<"git" | "folder">(hasGit ? "git" : "folder");
  let url = $state("");
  let folder = $state("");
  let name = $state("");
  let named = $state(false);
  let into = $state("");
  let writes = $state<Writes>("pull-request");
  let trusted = $state(false);
  let busy = $state(false);
  let error = $state<string | null>(null);

  /** "git@host:acme/team-prompts.git" → "team-prompts". */
  function slug(source: string): string {
    const last = source.trim().replace(/\/+$/, "").split(/[/:\\]/).pop() ?? "";
    return last
      .replace(/\.git$/, "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "");
  }

  $effect(() => {
    if (!named) name = slug(mode === "git" ? url : folder);
  });

  function setMode(next: "git" | "folder") {
    mode = next;
    writes = next === "git" ? "pull-request" : "commit";
    trusted = next === "folder";
    error = null;
  }

  async function chooseFolder() {
    const picked = await api.pickFolder("Choose a library folder", folder || "~/Prompts");
    if (picked) folder = picked;
  }

  async function chooseInto() {
    const picked = await api.pickFolder("Choose where to clone it", into || "~/Prompts");
    if (picked) into = `${picked.replace(/\/+$/, "")}/${name || "library"}`;
  }

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    busy = true;
    error = null;
    try {
      const added = await api.addLibrary({
        name,
        source: mode === "git" ? url : folder,
        path: mode === "git" && into.trim() ? into : null,
        writes,
        trusted,
      });
      onadded(added);
    } catch (err) {
      error = errorText(err);
    } finally {
      busy = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && oncancel()} />

<div class="dim" transition:dim|global role="presentation" onclick={() => !busy && oncancel()}></div>
<form transition:sheet|global class="sheet" aria-label="Add a library" onsubmit={submit}>
  <h3>Add a library</h3>
  <div class="sub">A team's git repo, or a folder of prompts on this computer.</div>
  <div class="seg" role="radiogroup" aria-label="Where the library comes from">
    <button type="button" role="radio" aria-checked={mode === "git"} class:on={mode === "git"} disabled={!hasGit} onclick={() => setMode("git")}>Git repo</button>
    <button type="button" role="radio" aria-checked={mode === "folder"} class:on={mode === "folder"} onclick={() => setMode("folder")}>Folder</button>
  </div>
  {#if mode === "git"}
    <label class="field">
      <span>Repo address</span>
      <input class="input mono" bind:value={url} placeholder="git@github.com:acme/prompts.git" spellcheck="false" required />
    </label>
    <div class="field">
      <span id="into">Clone into</span>
      <div class="field-row">
        <input class="input mono" bind:value={into} placeholder="~/Prompts/{name || 'name'}" aria-labelledby="into" spellcheck="false" />
        <button type="button" class="btn sm" onclick={() => void chooseInto()}>Choose…</button>
      </div>
    </div>
  {:else}
    <div class="field">
      <span id="folder">Folder</span>
      <div class="field-row">
        <input class="input mono" bind:value={folder} placeholder="~/Documents/prompts" aria-labelledby="folder" spellcheck="false" required />
        <button type="button" class="btn sm" onclick={() => void chooseFolder()}>Choose…</button>
      </div>
    </div>
  {/if}
  <label class="field">
    <span>Name, used in prompt IDs such as <code>{name || "acme"}/review</code></span>
    <input class="input" bind:value={name} oninput={() => (named = true)} placeholder="acme" spellcheck="false" required />
  </label>

  <div class="gh2">How your changes go in</div>
  <div class="grp choices">
    <label class="fr">
      <input type="radio" bind:group={writes} value="pull-request" />
      <span class="stack"><span>Pull request</span><span class="sub2">For team libraries. Each change is a proposal the team reviews.</span></span>
    </label>
    <label class="fr">
      <input type="radio" bind:group={writes} value="commit" />
      <span class="stack"><span>Commit directly</span><span class="sub2">For your own libraries. Sync commits and pushes.</span></span>
    </label>
  </div>
  <label class="chk trust">
    <input type="checkbox" bind:checked={trusted} />
    <span>Run this library's commands without asking. Off: a prompt that runs a command waits until you allow it.</span>
  </label>

  {#if error}<div class="banner err" role="alert"><Icon name="warn" small /><span class="grow">{error}</span></div>{/if}
  <div class="sfoot">
    <button type="button" class="btn" onclick={oncancel} disabled={busy}>Cancel</button>
    <button type="submit" class="btn primary" disabled={busy}>
      {busy ? (mode === "git" ? "Cloning…" : "Adding…") : "Add Library"}
    </button>
  </div>
</form>

<style>
  .seg {
    display: inline-flex;
    margin-top: 10px;
    padding: 2px;
    border-radius: 8px;
    background: var(--seg);
  }
  .seg button {
    padding: 3px 12px;
    border-radius: 6px;
    font-size: 12px;
    color: var(--ink2);
  }
  .seg button.on {
    background: var(--btn);
    color: var(--ink);
    box-shadow: inset 0 0 0 0.5px var(--edge);
  }
  .gh2 {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--ink2);
    margin: 12px 0 5px 2px;
  }
  .choices .fr {
    display: flex;
    gap: 9px;
    align-items: flex-start;
    padding: 7px 10px;
  }
  .choices .fr + .fr {
    border-top: 1px solid var(--rule);
  }
  .choices input {
    accent-color: var(--accent);
    margin: 2px 0 0;
  }
  .stack {
    display: flex;
    flex-direction: column;
  }
  .sub2 {
    font-size: 11.5px;
    color: var(--ink2);
  }
  .trust {
    margin-top: 10px;
    align-items: flex-start;
    color: var(--ink2);
  }
  .trust input {
    margin-top: 2px;
  }
  .banner {
    margin: 10px 0 0;
  }
</style>

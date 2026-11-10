<!-- Connect a library to a remote: paste the address of an empty repo, or
     create a private GitHub repo with gh. Nothing is created until the person
     clicks Create. -->
<script lang="ts">
  import { api, errorText, openLink } from "../lib/api";
  import type { Environment, SyncReport } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    library,
    env,
    ondone,
    oncancel,
  }: {
    library: string;
    env: Environment | null;
    ondone: (report: SyncReport) => void;
    oncancel: () => void;
  } = $props();

  let url = $state("");
  // svelte-ignore state_referenced_locally
  let repo = $state(library === "personal" ? "prompts" : library);
  let busy = $state<"url" | "gh" | null>(null);
  let error = $state<string | null>(null);
  const gh = $derived(env?.gh.state ?? "missing");

  async function run(kind: "url" | "gh") {
    busy = kind;
    error = null;
    try {
      const report =
        kind === "url" ? await api.connectRemote(library, url) : await api.createGithubRepo(library, repo);
      ondone(report);
    } catch (err) {
      error = errorText(err);
    } finally {
      busy = null;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && oncancel()} />

<div class="dim" role="presentation" onclick={() => !busy && oncancel()}></div>
<div class="sheet" role="dialog" aria-modal="true" aria-label="Connect {library} to a remote">
  <h3>Connect to a remote</h3>
  <div class="sub">Sync then keeps this library the same on every computer.</div>

  <form class="field" onsubmit={(e) => (e.preventDefault(), void run("url"))}>
    <span id="addr">Address of an empty repo</span>
    <div class="field-row">
      <input class="input mono" bind:value={url} aria-labelledby="addr" placeholder="git@github.com:you/prompts.git" spellcheck="false" required />
      <button class="btn sm" type="submit" disabled={busy !== null}>{busy === "url" ? "Connecting…" : "Connect"}</button>
    </div>
  </form>

  <div class="gh2">Or create one on GitHub</div>
  <div class="grp box">
    {#if gh === "signed-in"}
      <form class="field-row" onsubmit={(e) => (e.preventDefault(), void run("gh"))}>
        <span class="mono who">{env?.gh.account ?? "you"}/</span>
        <input class="input mono" bind:value={repo} aria-label="Repo name" spellcheck="false" required />
        <button class="btn sm primary" type="submit" disabled={busy !== null}>
          {busy === "gh" ? "Creating…" : "Create Private Repo"}
        </button>
      </form>
      <p class="fine">Creates a private repo with GitHub CLI, as {env?.gh.account ?? "you"}, then pushes this library.</p>
    {:else if gh === "signed-out"}
      <p class="fine first">
        GitHub CLI is not signed in. Sign in once with <code>gh auth login</code>, or make an empty
        private repo on github.com and paste its address above.
      </p>
      <div class="acts"><button class="btn sm" onclick={() => openLink("https://github.com/new")}>New Repo on github.com</button></div>
    {:else}
      <p class="fine first">
        With GitHub CLI, PromptShelf can create the repo for you. Or make an empty private repo on
        github.com and paste its address above.
      </p>
      <div class="acts">
        <button class="btn sm" onclick={() => openLink("https://cli.github.com")}>Get GitHub CLI</button>
        <button class="btn sm" onclick={() => openLink("https://github.com/new")}>New Repo on github.com</button>
      </div>
    {/if}
  </div>

  {#if error}<div class="banner err" role="alert"><Icon name="warn" small /><span class="grow">{error}</span></div>{/if}
  <div class="sfoot">
    <button class="btn" onclick={oncancel} disabled={busy !== null}>Cancel</button>
  </div>
</div>

<style>
  .gh2 {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--ink2);
    margin: 14px 0 5px 2px;
  }
  .box {
    padding: 9px 10px;
  }
  .who {
    color: var(--ink2);
    flex: none;
  }
  .fine.first {
    margin-top: 0;
  }
  .acts {
    display: flex;
    gap: 6px;
    margin-top: 8px;
  }
  .banner {
    margin: 10px 0 0;
  }
</style>

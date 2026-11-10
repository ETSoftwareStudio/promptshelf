<!-- Settings → Libraries: each library with its folder, remote, sync state and
     last error. Add, connect, change and remove libraries here. -->
<script lang="ts">
  import AddLibrarySheet from "../components/AddLibrarySheet.svelte";
  import ConfirmSheet from "../components/ConfirmSheet.svelte";
  import ConflictSheet from "../components/ConflictSheet.svelte";
  import ConnectSheet from "../components/ConnectSheet.svelte";
  import Icon from "../components/Icon.svelte";
  import { api, errorText, openLink } from "../lib/api";
  import { libraryLabel } from "../lib/format";
  import { syncState, syncSummary } from "../lib/nav";
  import type { Toast } from "../lib/toast.svelte";
  import type { Environment, LibraryInfo, LibraryStatus, Overview, SyncReport, Writes } from "../lib/types";

  let { toast }: { toast: Toast } = $props();

  let overview = $state<Overview | null>(null);
  let status = $state<LibraryStatus[]>([]);
  let env = $state<Environment | null>(null);
  let sheet = $state<
    { kind: "add" } | { kind: "connect" | "remove" | "conflict"; lib: LibraryInfo } | null
  >(null);
  let syncing = $state(false);
  let removing = $state(false);
  let who = $state({ name: "", email: "" });

  const noIdentity = $derived(Boolean(env?.git.version) && !(env?.git.name && env?.git.email));

  export async function load() {
    try {
      overview = await api.overview();
      [status, env] = await Promise.all([api.libraryStatus(), api.environment()]);
    } catch (e) {
      toast.show(errorText(e), true);
    }
  }

  void load();

  function shown(r: SyncReport) {
    const sum = syncSummary([r]);
    toast.show(sum.message ?? "Synced.", sum.bad, sum.bad ? 6000 : 2600);
  }

  async function syncNow() {
    syncing = true;
    try {
      const reports = await api.sync();
      const sum = syncSummary(reports);
      toast.show(sum.message ?? "Synced.", sum.bad, sum.bad ? 6000 : 2600);
    } catch (e) {
      toast.show(errorText(e), true);
    } finally {
      syncing = false;
      await load();
    }
  }

  async function change(lib: LibraryInfo, writes: Writes | null, trusted: boolean | null) {
    try {
      await api.updateLibrary(lib.name, writes, trusted);
      toast.show("Saved.");
    } catch (e) {
      toast.show(errorText(e), true);
    }
    await load();
  }

  async function remove(lib: LibraryInfo) {
    removing = true;
    try {
      const path = await api.removeLibrary(lib.name);
      toast.show(`Removed ${libraryLabel(lib.name)}. Its files stay in ${path}.`);
      sheet = null;
    } catch (e) {
      toast.show(errorText(e), true);
    } finally {
      removing = false;
      await load();
    }
  }

  async function saveIdentity(e: SubmitEvent) {
    e.preventDefault();
    try {
      env = await api.setGitIdentity(who.name, who.email);
      toast.show("Saved. New commits use your name.");
    } catch (err) {
      toast.show(errorText(err), true);
    }
  }
</script>

{#if env && !env.git.version}
  <div class="banner" role="alert">
    <Icon name="warn" small />
    <div class="grow">
      <p>Git is not installed. Prompts work without it; sync, remotes and team libraries need it.</p>
      <div class="acts"><button class="btn sm" onclick={() => openLink("https://git-scm.com/downloads")}>Get Git</button></div>
    </div>
  </div>
{:else if noIdentity}
  <form class="banner" onsubmit={saveIdentity}>
    <Icon name="warn" small />
    <div class="grow">
      <p>Git has no name and email for your commits, so they say “PromptShelf”. Add yours:</p>
      <div class="acts">
        <input class="input" bind:value={who.name} placeholder="Your name" aria-label="Your name" required />
        <input class="input" type="email" bind:value={who.email} placeholder="you@example.com" aria-label="Your email" required />
        <button class="btn sm" type="submit">Save</button>
      </div>
    </div>
  </form>
{/if}

{#if overview}
  <div class="grp">
    {#each overview.libraries as lib (lib.name)}
      {@const s = status.find((x) => x.name === lib.name)}
      {@const st = syncState(s, lib)}
      {@const last = s?.lastSync}
      <div class="fr col lib">
        <div class="top">
          <Icon name="folder" class="ico" />
          <span class="stack">
            <span class="nm">{libraryLabel(lib.name)}{#if lib.personal}<span class="tag">Personal</span>{/if}</span>
            <span class="sub2 mono" title={lib.path}>{lib.path}</span>
          </span>
          <select
            class="input pick"
            aria-label="How changes go into {libraryLabel(lib.name)}"
            value={lib.writes}
            disabled={lib.personal}
            onchange={(e) => void change(lib, e.currentTarget.value as Writes, null)}
          >
            <option value="commit">Commit directly</option>
            <option value="pull-request">Pull request</option>
          </select>
        </div>
        <div class="state">
          <span class="mono remote" title={s?.remote ?? ""}>{s?.remote ?? ""}</span>
          <span class="okt" class:bad={st.bad} class:off={!st.bad && !last}>
            {#if st.bad}<Icon name="warn" small />{:else if s?.remote && last}<Icon name="check" small />{/if}{st.text}
          </span>
        </div>
        {#if last?.conflicts.length}
          <div class="banner err">
            <Icon name="warn" small />
            <div class="grow">
              <p>Edited here and on the remote: <span class="mono">{last.conflicts.join(", ")}</span>.</p>
              <p>Your version stays in use until you choose what to keep.</p>
              <div class="acts"><button class="btn sm" onclick={() => (sheet = { kind: "conflict", lib })}>Choose…</button></div>
            </div>
          </div>
        {:else if last?.error}
          <div class="banner err">
            <Icon name="warn" small />
            <div class="grow">
              {#if last.hint}<p>{last.hint.message}</p>{/if}
              <p class="mono raw">{last.error}</p>
              {#if last.hint?.url}
                <div class="acts"><button class="btn sm" onclick={() => openLink(last.hint!.url!)}>How to Fix It</button></div>
              {/if}
            </div>
          </div>
        {/if}
        <div class="bottom">
          <label class="chk">
            <input type="checkbox" checked={lib.trusted} onchange={(e) => void change(lib, null, e.currentTarget.checked)} />
            Run its commands without asking
          </label>
          <span class="val">
            {#if !s?.remote && lib.exists && env?.git.version}
              <button class="btn sm" onclick={() => (sheet = { kind: "connect", lib })}>Connect…</button>
            {/if}
            {#if !lib.personal}
              <button class="btn sm danger" onclick={() => (sheet = { kind: "remove", lib })}>Remove…</button>
            {/if}
          </span>
        </div>
      </div>
    {:else}
      <div class="fr">No libraries yet.</div>
    {/each}
    <div class="fr">
      <button class="btn sm" onclick={() => (sheet = { kind: "add" })}><Icon name="plus" small />Add Library…</button>
      <span class="val">
        <button class="btn sm" onclick={() => void syncNow()} disabled={syncing}>{syncing ? "Syncing…" : "Sync Now"}</button>
      </span>
    </div>
  </div>
  <p class="foot">
    Team libraries use pull requests: your edits stay on this computer until you send them. Libraries
    are listed in <code>{overview.configPath}</code>.
  </p>
{/if}

{#if sheet?.kind === "add"}
  <AddLibrarySheet
    hasGit={Boolean(env?.git.version)}
    onadded={(name) => {
      sheet = null;
      toast.show(`Added ${libraryLabel(name)}.`);
      void load();
    }}
    oncancel={() => (sheet = null)}
  />
{:else if sheet?.kind === "connect"}
  <ConnectSheet
    library={sheet.lib.name}
    {env}
    ondone={(r) => {
      sheet = null;
      shown(r);
      void load();
    }}
    oncancel={() => (sheet = null)}
  />
{:else if sheet?.kind === "conflict"}
  {@const name = sheet.lib.name}
  <ConflictSheet
    library={name}
    files={status.find((x) => x.name === name)?.lastSync?.conflicts ?? []}
    ondone={(r) => {
      sheet = null;
      shown(r);
      void load();
    }}
    oncancel={() => (sheet = null)}
  />
{:else if sheet?.kind === "remove"}
  {@const lib = sheet.lib}
  <ConfirmSheet
    title="Remove {libraryLabel(lib.name)}?"
    message="PromptShelf stops showing it. The folder {lib.path} and its files stay on this computer."
    confirm="Remove"
    danger
    busy={removing}
    onconfirm={() => void remove(lib)}
    oncancel={() => (sheet = null)}
  />
{/if}

<style>
  .lib {
    gap: 4px;
  }
  .top,
  .state,
  .bottom {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }
  .nm {
    font-weight: 500;
  }
  .pick {
    width: auto;
    margin-left: auto;
    flex: none;
  }
  .state {
    padding-left: 25px;
    font-size: 11.5px;
  }
  .remote {
    color: var(--ink2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .okt {
    margin-left: auto;
    white-space: nowrap;
  }
  .okt.bad {
    color: var(--danger);
  }
  .bottom {
    padding-left: 25px;
  }
  .bottom .chk {
    color: var(--ink2);
  }
  .lib .banner {
    margin: 4px 0 2px 25px;
  }
  .raw {
    color: var(--ink2);
    font-size: 11px;
    overflow-wrap: anywhere;
  }
  .acts .input {
    width: 150px;
  }
</style>

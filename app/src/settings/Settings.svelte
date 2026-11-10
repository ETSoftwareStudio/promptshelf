<!-- Settings: the panel shortcut, the folder resolvers run in, and the
     libraries (read-only here; `shelf libs` changes them). -->
<script lang="ts">
  import { onMount } from "svelte";
  import Icon from "../components/Icon.svelte";
  import { api, errorText, onEvent } from "../lib/api";
  import { libraryLabel } from "../lib/format";
  import { syncSummary } from "../lib/nav";
  import { acceleratorFrom, displayAccelerator, isMac } from "../lib/hotkey";
  import { Toast } from "../lib/toast.svelte";
  import type { Overview, SettingsView } from "../lib/types";

  let tab = $state<"general" | "libraries">("general");
  let view = $state<SettingsView | null>(null);
  let overview = $state<Overview | null>(null);
  let recording = $state(false);
  let workDir = $state("");
  let error = $state<string | null>(null);
  let syncing = $state(false);
  const toast = new Toast();

  async function load() {
    try {
      view = await api.getSettings();
      workDir = view.settings.workDir;
      overview = await api.overview();
    } catch (e) {
      error = errorText(e);
    }
  }

  async function save(hotkey: string, dir: string) {
    if (!view) return;
    try {
      view = await api.setSettings({ hotkey, workDir: dir });
      error = null;
      toast.show("Saved.");
    } catch (e) {
      error = errorText(e);
    }
  }

  function record(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === "Escape") {
      recording = false;
      return;
    }
    if (e.key === "Backspace" || e.key === "Delete") {
      recording = false;
      void save("", workDir);
      return;
    }
    const accel = acceleratorFrom(e, isMac);
    if (accel) {
      recording = false;
      void save(accel, workDir);
    }
  }

  async function syncNow() {
    syncing = true;
    try {
      const reports = await api.sync();
      const sum = syncSummary(reports);
      toast.show(sum.message ?? "Synced.", sum.bad);
      overview = await api.overview();
    } catch (e) {
      toast.show(errorText(e), true);
    } finally {
      syncing = false;
    }
  }

  onMount(() => {
    void load();
    return onEvent("settings-opened", () => void load());
  });
</script>

<svelte:window onkeydown={record} />

<div class="set">
  <nav class="stabs" aria-label="Settings">
    <button class="stab" class:on={tab === "general"} onclick={() => (tab = "general")}>
      <Icon name="gear" />General
    </button>
    <button class="stab" class:on={tab === "libraries"} onclick={() => (tab = "libraries")}>
      <Icon name="folder" />Libraries
    </button>
  </nav>

  <div class="form">
    {#if error}<div class="banner" role="alert"><Icon name="warn" small />{error}</div>{/if}
    {#if view && tab === "general"}
      <div class="grp">
        <div class="fr">
          <span class="stack">
            <span>Quick panel shortcut</span>
            <span class="sub2">Opens the panel from any app.</span>
          </span>
          <span class="val">
            <button class="key" class:rec={recording} onclick={() => (recording = !recording)}>
              {recording ? "Press keys…" : displayAccelerator(view.settings.hotkey, isMac)}
            </button>
          </span>
        </div>
        {#if view.hotkeyError}
          <div class="fr warn"><Icon name="warn" small />{view.hotkeyError}</div>
        {/if}
        {#if recording}
          <div class="fr note">Press the new shortcut. Esc keeps the old one; Delete turns it off.</div>
        {/if}
      </div>

      <div class="grp">
        <div class="fr col">
          <span class="stack">
            <span>Run commands in</span>
            <span class="sub2">The folder for <code>branch</code> and command blanks, such as a repo. Empty means your home folder.</span>
          </span>
          <form class="dir" onsubmit={(e) => (e.preventDefault(), void save(view!.settings.hotkey, workDir))}>
            <input class="input mono" bind:value={workDir} placeholder="~/code/my-repo" spellcheck="false" />
            <button class="btn sm" type="submit" disabled={workDir === view.settings.workDir}>Save</button>
          </form>
        </div>
      </div>

      <div class="grp">
        <div class="fr">
          <span class="stack">
            <span>Paste into the front app</span>
            <span class="sub2">
              {view.pasteNote ?? (isMac ? "Copies, then presses ⌘V. macOS asks once for Accessibility." : "Copies, then presses Ctrl+V.")}
            </span>
          </span>
          <span class="val">{view.pasteNote ? "Copy only" : "On"}</span>
        </div>
      </div>
      <p class="foot">Settings live in <code>{view.settingsPath}</code>.</p>
    {:else if tab === "libraries" && overview}
      <div class="grp">
        {#each overview.libraries as lib (lib.name)}
          <div class="fr">
            <Icon name="folder" class="ico" />
            <span class="stack">
              <span>{libraryLabel(lib.name)}{#if lib.personal}<span class="tag">Personal</span>{/if}</span>
              <span class="sub2 mono" title={lib.path}>{lib.remote ?? lib.path}</span>
            </span>
            <span class="val">
              <span class="pop">{lib.writes === "commit" ? "Commit directly" : "Pull request"}</span>
              <span class="okt" class:off={!lib.git}>
                {#if lib.git}<Icon name="check" small />{lib.count} prompts{:else}Not a git repo{/if}
              </span>
            </span>
          </div>
        {:else}
          <div class="fr">No libraries yet.</div>
        {/each}
        <div class="fr">
          <span class="sub2">Add or change libraries with <code>shelf libs add</code> in a terminal.</span>
          <span class="val">
            <button class="btn sm" onclick={() => void syncNow()} disabled={syncing}>
              {syncing ? "Syncing…" : "Sync Now"}
            </button>
          </span>
        </div>
      </div>
      <p class="foot">Libraries are listed in <code>{overview.configPath}</code>.</p>
    {/if}
  </div>
</div>

{#if toast.text}<div class="toast" class:bad={toast.bad} role="status">{toast.text}</div>{/if}


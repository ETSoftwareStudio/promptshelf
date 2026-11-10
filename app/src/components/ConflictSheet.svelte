<!-- A sync found files edited here and on the remote. The sync stopped and
     this computer's version stays in use. The person picks what to keep. -->
<script lang="ts">
  import { dim, sheet } from "../lib/motion";
  import { api, errorText } from "../lib/api";
  import { libraryLabel } from "../lib/format";
  import type { Resolution, SyncReport } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    library,
    files,
    ondone,
    oncancel,
  }: {
    library: string;
    files: string[];
    ondone: (report: SyncReport) => void;
    oncancel: () => void;
  } = $props();

  let how = $state<Resolution>("both");
  let busy = $state(false);
  let error = $state<string | null>(null);

  const CHOICES: { how: Resolution; label: string; sub: string }[] = [
    {
      how: "both",
      label: "Keep both",
      sub: "Takes the remote's version and saves this computer's as a new prompt next to it, ending in -2.",
    },
    {
      how: "mine",
      label: "Keep this computer's version",
      sub: "Where both changed the same lines, yours win. Other edits from the remote stay.",
    },
    {
      how: "theirs",
      label: "Use the remote's version",
      sub: "Where both changed the same lines, the remote's win. Your other edits stay.",
    },
  ];

  async function resolve() {
    busy = true;
    error = null;
    try {
      const r = await api.resolveConflict(library, how);
      if (r.conflicts?.length || r.error) error = r.hint?.message ?? r.error ?? "The files still conflict.";
      else ondone(r);
    } catch (e) {
      error = errorText(e);
    } finally {
      busy = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && oncancel()} />

<div class="dim" transition:dim|global role="presentation" onclick={() => !busy && oncancel()}></div>
<div transition:sheet|global class="sheet" role="dialog" aria-modal="true" aria-label="Sync needs a look">
  <h3>{libraryLabel(library)}: edited here and on the remote</h3>
  <div class="sub">
    The sync stopped before it changed anything. This computer's version stays in use until you choose.
  </div>
  <ul class="files">
    {#each files as f (f)}<li class="mono"><Icon name="doc" small />{f}</li>{/each}
  </ul>
  <div class="grp choices" role="radiogroup" aria-label="What to keep">
    {#each CHOICES as c (c.how)}
      <label class="fr">
        <input type="radio" bind:group={how} value={c.how} />
        <span class="stack"><span>{c.label}</span><span class="sub2">{c.sub}</span></span>
      </label>
    {/each}
  </div>
  {#if error}<div class="banner err" role="alert"><Icon name="warn" small /><span class="grow">{error}</span></div>{/if}
  <div class="sfoot">
    <span class="grow fine">Nothing is force-pushed.</span>
    <button class="btn" onclick={oncancel} disabled={busy}>Later</button>
    <button class="btn primary" onclick={() => void resolve()} disabled={busy}>
      {busy ? "Syncing…" : "Resolve and Sync"}
    </button>
  </div>
</div>

<style>
  .files {
    list-style: none;
    margin: 10px 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .files li {
    display: flex;
    gap: 6px;
    align-items: center;
    color: var(--ink);
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
  .banner {
    margin: 10px 0 0;
  }
  .sfoot .fine {
    margin: 0;
  }
</style>

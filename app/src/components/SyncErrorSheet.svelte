<!-- The last sync of a library failed: what git said, in plain words first. -->
<script lang="ts">
  import { openLink } from "../lib/api";
  import { libraryLabel } from "../lib/format";
  import type { LastSync } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    library,
    last,
    onretry,
    onsettings,
    oncancel,
  }: {
    library: string;
    last: LastSync;
    onretry: () => void;
    onsettings: () => void;
    oncancel: () => void;
  } = $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && oncancel()} />

<div class="dim" role="presentation" onclick={oncancel}></div>
<div class="sheet" role="alertdialog" aria-modal="true" aria-label="The sync failed">
  <h3>{libraryLabel(library)} did not sync</h3>
  <div class="sub">Nothing was lost. Your prompts stay as they are on this computer.</div>
  <div class="banner err">
    <Icon name="warn" small />
    <div class="grow">
      {#if last.hint}<p>{last.hint.message}</p>{/if}
      <p class="mono raw">{last.error}</p>
    </div>
  </div>
  <div class="sfoot">
    {#if last.hint?.url}
      <button class="btn grow-left" onclick={() => openLink(last.hint!.url!)}>How to Fix It</button>
    {/if}
    <button class="btn" onclick={onsettings}>Library Settings</button>
    <button class="btn" onclick={oncancel}>Close</button>
    <button class="btn primary" onclick={onretry}>Try Again</button>
  </div>
</div>

<style>
  .banner {
    margin: 10px 0 0;
  }
  .raw {
    color: var(--ink2);
    font-size: 11px;
    overflow-wrap: anywhere;
  }
  .grow-left {
    margin-right: auto;
  }
</style>

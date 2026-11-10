<!-- Settings → About → Third-party Licences: what the app ships, made at
     build time by scripts/third-party.mjs. MPL-2.0 rows link to the source
     code of that exact version, as MPL-2.0 asks. -->
<script lang="ts">
  import type { ThirdParty } from "virtual:third-party";
  import Icon from "../components/Icon.svelte";
  import { openLink } from "../lib/api";

  let { onback }: { onback: () => void } = $props();

  type List = { crates: ThirdParty[]; npm: ThirdParty[]; fonts: ThirdParty[]; error?: string };
  let list = $state<List | null>(null);
  let failed = $state<string | null>(null);

  import("virtual:third-party").then(
    (m) => (list = m.default),
    (e) => (failed = String(e)),
  );

  const groups = $derived(
    list
      ? [
          { label: "Rust crates", rows: list.crates },
          { label: "npm packages", rows: list.npm },
          { label: "Fonts", rows: list.fonts },
        ].filter((g) => g.rows.length)
      : [],
  );
</script>

<div class="head">
  <button class="btn sm" onclick={onback}><Icon name="back" small />About</button>
  <h2>Third-party Licences</h2>
</div>
<p class="foot">
  PromptShelf is MIT. It ships the open-source code below, unchanged, under each one’s own licence. For MPL-2.0 code, Source
  opens the source code of that exact version.
</p>

{#if failed || list?.error}
  <div class="banner" role="alert"><Icon name="warn" small />The list could not be made: {failed ?? list?.error}</div>
{/if}

{#each groups as g (g.label)}
  <div class="gh">{g.label} <span class="n">{g.rows.length}</span></div>
  <div class="grp">
    {#each g.rows as r (`${r.name}@${r.version ?? ""}`)}
      <div class="fr row">
        <span class="name">{r.name}</span>
        {#if r.version}<span class="v">{r.version}</span>{/if}
        <span class="val">
          {r.licence}
          {#if r.source}
            <button class="linkish" onclick={() => openLink(r.source!)}>Source</button>
          {/if}
        </span>
      </div>
    {/each}
  </div>
{/each}

<style>
  .head {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }
  .foot {
    margin: 0 2px;
  }
  .n {
    font-weight: 400;
    color: var(--ink3);
    margin-left: 4px;
  }
  .row {
    min-height: 28px;
    padding: 4px 12px;
    font-size: 12px;
    gap: 8px;
    user-select: text;
    -webkit-user-select: text;
  }
  .name {
    font-family: var(--font-mono);
    font-size: 11.5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .v {
    color: var(--ink3);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    flex: none;
  }
  .val {
    font-size: 11.5px;
  }
</style>

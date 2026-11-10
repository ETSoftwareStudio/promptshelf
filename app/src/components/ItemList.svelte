<!-- The middle column for the Inbox, proposals and files to fix: groups of
     rows, with arrow keys. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Group } from "../lib/types";

  let {
    title,
    groups,
    selected,
    empty,
    loading = false,
    tools,
    onselect,
  }: {
    title: string;
    groups: Group[];
    selected: string | null;
    empty: string;
    loading?: boolean;
    tools?: Snippet;
    onselect: (key: string) => void;
  } = $props();

  let rows = $state<HTMLElement>();
  const all = $derived(groups.flatMap((g) => g.rows));

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    if (!all.length) return;
    const i = all.findIndex((r) => r.key === selected);
    const by = e.key === "ArrowDown" ? 1 : -1;
    onselect(all[Math.max(0, Math.min(all.length - 1, (i < 0 ? -1 : i) + by))].key);
    requestAnimationFrame(() => rows?.querySelector(".row.on")?.scrollIntoView({ block: "nearest" }));
  }
</script>

<section class="list">
  <header class="lh" data-tauri-drag-region>
    <b data-tauri-drag-region>{title}</b>
    {@render tools?.()}
  </header>
  <div class="rows" bind:this={rows} role="listbox" tabindex="0" aria-label={title} {onkeydown}>
    {#each groups as g (g.label)}
      {#if g.rows.length}
        {#if groups.length > 1}<div class="gl">{g.label}</div>{/if}
        {#each g.rows as r (r.key)}
          <button
            class="row"
            class:on={r.key === selected}
            role="option"
            aria-selected={r.key === selected}
            tabindex="-1"
            onclick={() => onselect(r.key)}
          >
            <span class="t">{r.title}</span>
            <span class="w">{r.when ?? ""}</span>
            <span class="d">
              {#if r.chip}<span class="chip {r.tone ?? ''}">{r.chip}</span>{/if}
              <span class="desc">{r.sub}</span>
            </span>
          </button>
        {/each}
      {/if}
    {/each}
    {#if !all.length && !loading}<p class="none">{empty}</p>{/if}
  </div>
</section>

<style>
  .list {
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--rule);
    min-width: 0;
    min-height: 0;
    background: var(--ground);
  }
  .lh {
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    padding: 0 10px 0 16px;
    flex: none;
  }
  .lh b {
    font-size: 15px;
    font-weight: 650;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .rows {
    padding: 0 8px 8px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    outline: none;
  }
  .gl {
    font-size: 11px;
    font-weight: 600;
    color: var(--ink3);
    padding: 8px 10px 3px;
  }
  .row {
    padding: 7px 10px;
    border-radius: 8px;
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 1px 8px;
    width: 100%;
    text-align: left;
    font-size: 13px;
  }
  .t {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  .w {
    font-size: 11.5px;
    color: var(--ink3);
  }
  .d {
    grid-column: 1 / -1;
    font-size: 12px;
    color: var(--ink2);
    white-space: nowrap;
    overflow: hidden;
    display: flex;
    gap: 5px;
    align-items: center;
    min-width: 0;
  }
  .desc {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .row.on {
    background: var(--accent);
    color: var(--on-accent);
  }
  .row.on .w,
  .row.on .d {
    color: var(--on-accent-dim);
  }
  .row.on .chip {
    background: oklch(100% 0 0 / 0.2);
    color: var(--on-accent);
  }
  .rows:focus-visible .row.on {
    outline: 2px solid var(--focus);
    outline-offset: 1px;
  }
  .none {
    margin: 24px 8px;
    text-align: center;
    color: var(--ink3);
    font-size: 12.5px;
  }
</style>

<!-- The middle column: a title, search, and the prompts. -->
<script lang="ts">
  import { libraryLabel, shortDay } from "../lib/format";
  import type { Summary } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    title,
    items,
    selected,
    query = $bindable(""),
    loading = false,
    personal = null,
    searchEl = $bindable(),
    onselect,
    onnew,
    onopen,
  }: {
    title: string;
    items: Summary[];
    selected: string | null;
    query?: string;
    loading?: boolean;
    /** The Personal library's name; other libraries show a tag. */
    personal?: string | null;
    searchEl?: HTMLInputElement;
    onselect: (id: string) => void;
    onnew: () => void;
    /** Enter on the list: open the selected prompt's Send preview. */
    onopen: () => void;
  } = $props();

  let rows = $state<HTMLElement>();

  function move(by: number) {
    if (!items.length) return;
    const i = items.findIndex((s) => s.fullId === selected);
    const next = Math.max(0, Math.min(items.length - 1, (i < 0 ? -1 : i) + by));
    onselect(items[next].fullId);
    requestAnimationFrame(() => {
      rows?.querySelector(".row.on")?.scrollIntoView({ block: "nearest" });
    });
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      move(1);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      move(-1);
    } else if (e.key === "Enter" && !e.metaKey && !e.ctrlKey) {
      e.preventDefault();
      onopen();
    } else if (e.key === "Escape" && query) {
      e.preventDefault();
      query = "";
    }
  }
</script>

<section class="list">
  <header class="lh" data-tauri-drag-region>
    <b data-tauri-drag-region>{title}</b>
    <button class="btn icon" title="New prompt" aria-label="New prompt" onclick={onnew}>
      <Icon name="plus" />
    </button>
  </header>
  <div class="find">
    <label class="search">
      <Icon name="search" small />
      <input
        bind:this={searchEl}
        bind:value={query}
        placeholder="Search prompts"
        spellcheck="false"
        aria-label="Search prompts"
        {onkeydown}
      />
    </label>
  </div>
  <div class="rows" bind:this={rows} role="listbox" tabindex="-1" aria-label={title} {onkeydown}>
    {#each items as s (s.fullId)}
      <button
        class="row"
        class:on={s.fullId === selected}
        role="option"
        aria-selected={s.fullId === selected}
        onclick={() => onselect(s.fullId)}
        ondblclick={onopen}
      >
        <span class="t">
          {#if s.pin}<Icon name="pin" small />{/if}
          <span>{s.title}</span>
        </span>
        <span class="w">{shortDay(s.lastUsed)}</span>
        <span class="d">
          {#if personal && s.library !== personal}<span class="lib"
              >{libraryLabel(s.library)}</span
            >{/if}
          <span class="desc">{s.description || s.fullId}</span>
        </span>
      </button>
    {:else}
      {#if !loading}
        <p class="none">{query ? `Nothing matches “${query}”.` : "No prompts here yet."}</p>
      {/if}
    {/each}
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
  .find {
    padding: 0 8px 8px;
  }
  .rows {
    padding: 0 8px 8px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    outline: none;
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
    display: flex;
    align-items: center;
    gap: 5px;
    min-width: 0;
  }
  .t span {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .t :global(.i) {
    color: var(--pin);
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
  .lib {
    flex: none;
    font-size: 10.5px;
    font-weight: 600;
    padding: 0 5px;
    border-radius: 5px;
    background: var(--seg);
    color: var(--ink2);
    line-height: 16px;
  }
  .row.on {
    background: var(--accent);
    color: var(--on-accent);
  }
  .row.on .w,
  .row.on .d,
  .row.on .t :global(.i) {
    color: var(--on-accent-dim);
  }
  .row.on .lib {
    background: oklch(100% 0 0 / 0.2);
    color: var(--on-accent);
  }
  .none {
    margin: 24px 8px;
    text-align: center;
    color: var(--ink3);
    font-size: 12.5px;
  }
</style>

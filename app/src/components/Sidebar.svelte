<!-- The sidebar: pins (⌘1 to ⌘9), prompts, kinds, libraries and sync. -->
<script lang="ts">
  import { KINDS, libraryLabel } from "../lib/format";
  import { modKey } from "../lib/hotkey";
  import type { Overview } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    overview,
    nav,
    pinned = null,
    syncing = false,
    syncNote = "",
    onnav,
    onpin,
    onsync,
  }: {
    overview: Overview;
    nav: string;
    /** The full ID of the prompt open from a pin, to mark it. */
    pinned?: string | null;
    syncing?: boolean;
    syncNote?: string;
    onnav: (nav: string) => void;
    onpin: (fullId: string) => void;
    onsync: () => void;
  } = $props();

  const count = (kind: string) => overview.kinds.find((k) => k.kind === kind)?.count ?? 0;
  const drafts = $derived(count("draft"));
  const kinds = $derived(KINDS.filter((k) => k.kind === "template" || count(k.kind) > 0));
  const anyGit = $derived(overview.libraries.some((l) => l.git));
</script>

<aside class="side">
  <div class="drag" data-tauri-drag-region></div>
  {#if overview.pins.length}
    <div class="sh first">Pinned</div>
    {#each overview.pins as pin (pin.fullId)}
      <button
        class="si"
        class:on={pinned === pin.fullId}
        class:gone={!pin.exists}
        title={pin.exists ? pin.fullId : `${pin.fullId} no longer exists`}
        onclick={() => onpin(pin.fullId)}
      >
        <Icon name="pin" class="p" />
        <span class="label">{pin.title}</span>
        <span class="n kbd">{modKey(String(pin.position))}</span>
      </button>
    {/each}
  {/if}

  <div class="sh" class:first={!overview.pins.length}>Prompts</div>
  <button class="si" class:on={nav === "all" && !pinned} onclick={() => onnav("all")}>
    <Icon name="doc" /><span class="label">All Prompts</span><span class="n">{overview.total}</span>
  </button>
  <button class="si" class:on={nav === "recent"} onclick={() => onnav("recent")}>
    <Icon name="clock" /><span class="label">Recent</span>
  </button>
  <button class="si" class:on={nav === "kind:draft"} onclick={() => onnav("kind:draft")}>
    <Icon name="tray" /><span class="label">Inbox</span>
    {#if drafts}<span class="n"><span class="badge">{drafts}</span></span>{/if}
  </button>

  <div class="sh">Kinds</div>
  {#each kinds as k (k.kind)}
    <button class="si" class:on={nav === `kind:${k.kind}`} onclick={() => onnav(`kind:${k.kind}`)}>
      <Icon name={k.icon} /><span class="label">{k.label}</span><span class="n">{count(k.kind)}</span>
    </button>
  {/each}

  <div class="sh">Libraries</div>
  {#each overview.libraries as lib (lib.name)}
    <button
      class="si"
      class:on={nav === `lib:${lib.name}`}
      title={lib.path}
      onclick={() => onnav(`lib:${lib.name}`)}
    >
      <Icon name="folder" /><span class="label">{libraryLabel(lib.name)}</span>
      <span class="n">{lib.count}</span>
      <span
        class="ld"
        class:off={!lib.git}
        class:bad={!lib.exists}
        title={!lib.exists ? "Folder missing" : lib.git ? "A git repo" : "This computer only"}
      ></span>
    </button>
  {/each}

  <button class="sync" onclick={onsync} disabled={syncing || !anyGit} title="Sync every library">
    <span class="dot" class:off={!anyGit} class:busy={syncing}></span>
    <span class="label">
      {#if syncing}Syncing…{:else if !anyGit}This computer only{:else}{syncNote || "Sync now"}{/if}
    </span>
    {#if anyGit && !syncing}<Icon name="sync" small />{/if}
  </button>
</aside>

<style>
  .side {
    margin: 8px 0 8px 8px;
    border-radius: 14px;
    background: var(--side);
    padding: 12px 8px 10px;
    box-shadow: inset 0 0 0 0.5px var(--edge);
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
  }
  :global(html.vibrant) .side {
    background: color-mix(in oklch, var(--side) 55%, transparent);
  }
  :global(html.mac) .drag {
    height: 26px;
    flex: none;
  }
  .sh {
    font-size: 11px;
    font-weight: 600;
    color: var(--ink3);
    padding: 10px 8px 4px;
  }
  .sh.first {
    padding-top: 2px;
  }
  .si {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border-radius: 7px;
    white-space: nowrap;
    text-align: left;
    width: 100%;
    font-size: 13px;
  }
  .si :global(.i) {
    color: var(--link);
  }
  .si :global(.i.p) {
    color: var(--pin);
  }
  .si.gone {
    color: var(--ink3);
  }
  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  .n {
    margin-left: auto;
    font-size: 11.5px;
    color: var(--ink3);
    flex: none;
  }
  .si.on {
    background: var(--side-sel);
    font-weight: 500;
  }
  .badge {
    display: inline-block;
    min-width: 18px;
    padding: 0 6px;
    border-radius: 9px;
    font-size: 11px;
    font-weight: 600;
    text-align: center;
    line-height: 17px;
    background: var(--badge);
    color: var(--on-accent);
  }
  .ld {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--ok);
    flex: none;
    margin-left: 2px;
  }
  .ld.off {
    background: var(--ink3);
  }
  .ld.bad {
    background: var(--warn);
  }
  .sync {
    margin-top: auto;
    padding: 8px 8px 0;
    border-top: 1px solid var(--rule);
    font-size: 11px;
    color: var(--ink2);
    display: flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
    text-align: left;
    width: 100%;
  }
  .sync:not(:disabled):hover {
    color: var(--ink);
  }
  .sync :global(.i) {
    margin-left: auto;
    color: var(--ink3);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--ok);
    flex: none;
  }
  .dot.off {
    background: var(--ink3);
  }
  .dot.busy {
    animation: pulse 1s ease-in-out infinite alternate;
  }
  @keyframes pulse {
    to {
      opacity: 0.3;
    }
  }
</style>

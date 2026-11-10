<!-- The library window: sidebar | list | detail. -->
<script lang="ts">
  import { onMount } from "svelte";
  import Detail from "../components/Detail.svelte";
  import FirstRun from "../components/FirstRun.svelte";
  import NoPrompt from "../components/NoPrompt.svelte";
  import NewPromptSheet from "../components/NewPromptSheet.svelte";
  import PromptList from "../components/PromptList.svelte";
  import SendSheet from "../components/SendSheet.svelte";
  import Sidebar from "../components/Sidebar.svelte";
  import { api, copyInBrowser, errorText, onEvent } from "../lib/api";
  import { Filler } from "../lib/filler.svelte";
  import { libraryKey } from "../lib/keys";
  import { navQuery, navTitle, syncSummary } from "../lib/nav";
  import { onFocus } from "../lib/platform";
  import { Toast } from "../lib/toast.svelte";
  import type { Detail as DetailT, Kind, Overview, Summary } from "../lib/types";

  let overview = $state<Overview | null>(null);
  let loadError = $state<string | null>(null);
  /** "all", "recent", "kind:<kind>" or "lib:<name>". */
  let nav = $state("all");
  let query = $state("");
  let items = $state<Summary[]>([]);
  let listing = $state(false);
  let selected = $state<string | null>(null);
  /** The prompt opened from a pin, to mark the pin in the sidebar. */
  let fromPin = $state<string | null>(null);
  let detail = $state<DetailT | null>(null);
  let editing = $state(false);
  let sheet = $state<"send" | "new" | null>(null);
  let sending = $state(false);
  let syncing = $state(false);
  let syncNote = $state("");
  let searchEl = $state<HTMLInputElement>();
  let detailView = $state<ReturnType<typeof Detail>>();
  const filler = new Filler();
  const toast = new Toast();

  const title = $derived(navTitle(nav));

  async function loadOverview() {
    try {
      overview = await api.overview();
      loadError = null;
    } catch (e) {
      loadError = errorText(e);
    }
  }

  let listSeq = 0;
  async function loadList(keep = true) {
    if (!overview?.hasConfig) return;
    const seq = ++listSeq;
    listing = true;
    try {
      const out = await api.list(navQuery(nav, query));
      if (seq !== listSeq) return;
      items = out;
      if (!keep || !out.some((s) => s.fullId === selected)) {
        await select(out[0]?.fullId ?? null);
      }
    } catch (e) {
      if (seq === listSeq) toast.show(errorText(e), true);
    } finally {
      if (seq === listSeq) listing = false;
    }
  }

  async function select(id: string | null, pin = false) {
    if (pin) fromPin = id;
    else if (id !== fromPin) fromPin = null;
    if (id === selected && detail?.fullId === id) return;
    selected = id;
    editing = false;
    if (!id) {
      detail = null;
      filler.clear();
      return;
    }
    try {
      const d = await api.get(id);
      if (selected !== id) return;
      detail = d;
      void filler.load(id);
    } catch (e) {
      toast.show(errorText(e), true);
    }
  }

  /** Reads the files again and keeps the selection. */
  async function refresh() {
    await loadOverview();
    await loadList();
    if (selected && !editing) {
      try {
        detail = await api.get(selected);
      } catch {
        await select(items[0]?.fullId ?? null);
      }
    }
  }

  function setNav(next: string) {
    nav = next;
    fromPin = null;
    void loadList(false);
  }

  /** Clears the search without the search-as-you-type reload, which would
   *  move the selection. */
  function clearQuery() {
    clearTimeout(timer);
    query = "";
    lastQuery = "";
  }

  async function openPin(fullId: string) {
    if (!items.some((s) => s.fullId === fullId)) {
      nav = "all";
      clearQuery();
      await loadList();
    }
    await select(fullId, true);
  }

  async function togglePin() {
    if (!detail) return;
    try {
      if (detail.pin) await api.unpin(detail.fullId);
      else await api.pin(detail.fullId);
      await loadOverview();
      await loadList();
      detail = await api.get(detail.fullId);
    } catch (e) {
      toast.show(errorText(e), true);
    }
  }

  async function copy() {
    if (!detail) return;
    sending = true;
    try {
      const r = await api.send(detail.fullId, filler.values());
      let copied = r.copied;
      if (!copied) copied = await copyInBrowser(r.text);
      sheet = null;
      toast.show(copied ? "Copied. Paste it into any chat." : (r.message ?? "Could not copy."), !copied);
      detail = await api.get(detail.fullId);
    } catch (e) {
      toast.show(errorText(e), true);
    } finally {
      sending = false;
    }
  }

  async function create(title: string, kind: Kind, library: string) {
    sheet = null;
    try {
      const id = await api.newPrompt(title, kind, library);
      await loadOverview();
      nav = "all";
      clearQuery();
      await loadList();
      await select(id);
      detailView?.edit();
    } catch (e) {
      toast.show(errorText(e), true);
    }
  }

  async function saved(id: string) {
    toast.show("Saved.");
    await loadOverview();
    await loadList();
    selected = null;
    await select(id);
  }

  async function sync() {
    if (syncing) return;
    syncing = true;
    try {
      const reports = await api.sync();
      const sum = syncSummary(reports);
      const time = new Date().toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
      syncNote = sum.bad ? "Sync needs a look" : `Synced at ${time}`;
      if (sum.message) toast.show(sum.message, sum.bad, sum.bad ? 6000 : 2600);
      await refresh();
    } catch (e) {
      toast.show(errorText(e), true);
    } finally {
      syncing = false;
    }
  }

  async function started(path: string) {
    toast.show(`Your Personal library is in ${path}.`, false, 4000);
    await loadOverview();
    await loadList(false);
  }

  // Search as you type.
  let timer: ReturnType<typeof setTimeout> | undefined;
  let lastQuery = "";
  $effect(() => {
    const q = query;
    if (q === lastQuery) return;
    lastQuery = q;
    clearTimeout(timer);
    timer = setTimeout(() => void loadList(false), 90);
  });

  function onkeydown(e: KeyboardEvent) {
    const key = sheet ? null : libraryKey(e);
    if (!key) return;
    if (typeof key === "object") {
      const pin = overview?.pins[key.pin - 1];
      if (!pin) return;
      void openPin(pin.fullId);
    } else if (key === "find") {
      searchEl?.focus();
      searchEl?.select();
    } else if (key === "new") {
      sheet = "new";
    } else if (!detail || editing) {
      return;
    } else if (key === "send") {
      sheet = "send";
    } else {
      detailView?.edit();
    }
    e.preventDefault();
  }

  onMount(() => {
    void loadOverview().then(() => loadList(false));
    const stops = [
      onFocus(() => void refresh()),
      onEvent<string>("menu", (id) => {
        if (id === "new-prompt") sheet = "new";
        else if (id === "sync") void sync();
        else if (id === "find") searchEl?.focus();
      }),
    ];
    return () => stops.forEach((s) => s());
  });
</script>

<svelte:window {onkeydown} />

{#if !overview}
  <div class="loading" data-tauri-drag-region>
    {#if loadError}<div class="empty"><h4>PromptShelf could not start</h4><p>{loadError}</p></div>{/if}
  </div>
{:else if !overview.hasConfig}
  <FirstRun ondone={(path) => void started(path)} />
{:else}
  <main class="lib">
    <Sidebar
      {overview}
      {nav}
      pinned={fromPin}
      {syncing}
      {syncNote}
      onnav={setNav}
      onpin={(id) => void openPin(id)}
      onsync={() => void sync()}
    />
    <PromptList
      {title}
      {items}
      {selected}
      bind:query
      bind:searchEl
      loading={listing}
      personal={overview.personal}
      onselect={(id) => void select(id)}
      onnew={() => (sheet = "new")}
      onopen={() => detail && (sheet = "send")}
    />
    {#if detail}
      <Detail
        bind:this={detailView}
        bind:editing
        {detail}
        {filler}
        onpin={() => void togglePin()}
        onsend={() => (sheet = "send")}
        onsaved={(id) => void saved(id)}
      />
    {:else}
      <NoPrompt problems={overview.diagnostics} />
    {/if}
  </main>
{/if}

{#if sheet === "send" && detail && filler.view}
  <SendSheet
    title={detail.title}
    text={filler.view.sendText}
    open={filler.view.missing.length}
    busy={sending}
    oncopy={() => void copy()}
    oncancel={() => (sheet = null)}
  />
{:else if sheet === "new" && overview}
  <NewPromptSheet
    libraries={overview.libraries}
    personal={overview.personal}
    oncreate={(t, k, l) => void create(t, k, l)}
    oncancel={() => (sheet = null)}
  />
{/if}

{#if toast.text}<div class="toast" class:bad={toast.bad} role="status">{toast.text}</div>{/if}

<style>
  .lib {
    display: grid;
    grid-template-columns: minmax(176px, 200px) minmax(220px, 280px) minmax(0, 1fr);
    height: 100%;
  }
  .loading {
    height: 100%;
    background: var(--ground);
  }
</style>

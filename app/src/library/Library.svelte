<!-- The library window: sidebar | list | detail. The Inbox, proposals and
     files to fix have their own lists. -->
<script lang="ts">
  import { rise } from "../lib/motion";
  import { onMount } from "svelte";
  import AddLibrarySheet from "../components/AddLibrarySheet.svelte";
  import ConflictSheet from "../components/ConflictSheet.svelte";
  import Detail from "../components/Detail.svelte";
  import FirstRun from "../components/FirstRun.svelte";
  import Icon from "../components/Icon.svelte";
  import ItemList from "../components/ItemList.svelte";
  import NoPrompt from "../components/NoPrompt.svelte";
  import NewPromptSheet from "../components/NewPromptSheet.svelte";
  import ProblemDetail from "../components/ProblemDetail.svelte";
  import PromptList from "../components/PromptList.svelte";
  import SendSheet from "../components/SendSheet.svelte";
  import Sidebar from "../components/Sidebar.svelte";
  import SyncErrorSheet from "../components/SyncErrorSheet.svelte";
  import UpdateDetail from "../components/UpdateDetail.svelte";
  import { api, copyInBrowser, errorText, onEvent } from "../lib/api";
  import { Filler } from "../lib/filler.svelte";
  import { libraryKey } from "../lib/keys";
  import { inboxGroups, problemGroups, proposalGroups, updateKey } from "../lib/lists";
  import { navQuery, navTitle, syncSummary } from "../lib/nav";
  import { onFocus } from "../lib/platform";
  import { Toast } from "../lib/toast.svelte";
  import type {
    Detail as DetailT,
    InboxView,
    Kind,
    Overview,
    ProposalView,
    Summary,
    SyncReport,
  } from "../lib/types";
  import PromptActions from "./PromptActions.svelte";

  let overview = $state<Overview | null>(null);
  let loadError = $state<string | null>(null);
  /** "all", "recent", "inbox", "proposals", "problems", "kind:<kind>" or "lib:<name>". */
  let nav = $state("all");
  let query = $state("");
  let items = $state<Summary[]>([]);
  let inbox = $state<InboxView | null>(null);
  let proposals = $state<ProposalView[]>([]);
  let listing = $state(false);
  /** A prompt's full ID, an update's key, or a broken file's path. */
  let selected = $state<string | null>(null);
  /** The prompt opened from a pin, to mark the pin in the sidebar. */
  let fromPin = $state<string | null>(null);
  let detail = $state<DetailT | null>(null);
  let editing = $state(false);
  let sheet = $state<"send" | "new" | "addlib" | null>(null);
  /** The library whose sync failed or found a conflict, for its sheet. */
  let syncSheet = $state<string | null>(null);
  let sending = $state(false);
  let syncing = $state(false);
  let refreshing = $state(false);
  let syncNote = $state("");
  let searchEl = $state<HTMLInputElement>();
  let detailView = $state<ReturnType<typeof Detail>>();
  let promptActions = $state<ReturnType<typeof PromptActions>>();
  const filler = new Filler();
  const toast = new Toast();

  const title = $derived(navTitle(nav));
  const special = $derived(nav === "inbox" || nav === "proposals" || nav === "problems");
  const groups = $derived(
    nav === "inbox"
      ? inboxGroups(inbox)
      : nav === "proposals"
        ? proposalGroups(proposals)
        : problemGroups(overview?.diagnostics ?? []),
  );
  const keys = $derived(groups.flatMap((g) => g.rows.map((r) => r.key)));
  const update = $derived(inbox?.updates.find((u) => updateKey(u.fullId) === selected) ?? null);
  const problem = $derived(
    nav === "problems" ? (overview?.diagnostics.find((d) => d.path === selected) ?? null) : null,
  );
  const syncLib = $derived(overview?.libraries.find((l) => l.name === syncSheet) ?? null);

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
      let first: string | null = null;
      let has = (key: string | null) => items.some((s) => s.fullId === key);
      if (nav === "inbox") {
        inbox = await api.inbox();
        has = (key) => keys.includes(key ?? "");
        first = keys[0] ?? null;
      } else if (nav === "proposals") {
        proposals = await api.proposals();
        has = (key) => proposals.some((p) => p.fullId === key);
        first = proposals[0]?.fullId ?? null;
      } else if (nav === "problems") {
        has = (key) => keys.includes(key ?? "");
        first = keys[0] ?? null;
      } else {
        const out = await api.list(navQuery(nav, query));
        if (seq !== listSeq) return;
        items = out;
        first = out[0]?.fullId ?? null;
      }
      if (seq !== listSeq) return;
      if (!keep || !has(selected)) await select(first);
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
    if (!id || id.startsWith("update:") || nav === "problems") {
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
      detail = null;
      toast.show(errorText(e), true);
    }
  }

  /** Reads the files again and keeps the selection. */
  async function refresh() {
    await loadOverview();
    await loadList();
    if (detail && selected === detail.fullId && !editing) {
      try {
        detail = await api.get(detail.fullId);
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

  async function show(id: string | null | undefined) {
    if (!id) {
      await loadList(false);
      return;
    }
    if (!special && !items.some((s) => s.fullId === id)) {
      nav = "all";
      clearQuery();
      await loadList();
    }
    selected = null;
    await select(id);
  }

  async function openPin(fullId: string) {
    if (special || !items.some((s) => s.fullId === fullId)) {
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
    const team = detail?.writes === "pull-request" && !detail.proposalInfo;
    toast.show(team ? "Saved as your proposal. Send it when it is ready." : "Saved.");
    await loadOverview();
    await loadList();
    selected = null;
    await select(id);
  }

  async function changed(next: string | null | undefined, message: string) {
    toast.show(message, false, 4000);
    await loadOverview();
    await show(next);
  }

  function synced(reports: SyncReport[]) {
    const sum = syncSummary(reports);
    const time = new Date().toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
    syncNote = sum.bad ? "" : `Synced at ${time}`;
    const conflict = reports.find((r) => r.conflicts?.length);
    if (conflict) syncSheet = conflict.library;
    else if (sum.message) toast.show(sum.message, sum.bad, sum.bad ? 6000 : 2600);
  }

  async function sync() {
    if (syncing) return;
    syncing = true;
    syncSheet = null;
    try {
      synced(await api.sync());
      await refresh();
    } catch (e) {
      toast.show(errorText(e), true);
    } finally {
      syncing = false;
    }
  }

  async function refreshProposals() {
    refreshing = true;
    try {
      proposals = await api.proposals(true);
      await refresh();
    } catch (e) {
      toast.show(errorText(e), true);
    } finally {
      refreshing = false;
    }
  }

  async function onUpdate(action: "allow" | "keep" | "copy" | "done") {
    if (!update) return;
    const id = update.fullId;
    try {
      if (action === "allow") {
        const allowed = await api.allowCommands(id);
        toast.show(`Allowed ${allowed.join(", ")}.`);
      } else if (action === "keep") {
        const c = await api.keepPrevious(id);
        toast.show(`Kept the previous text as ${c.id}.`, false, 4000);
      } else if (action === "copy") {
        const c = await api.copyPrompt(id);
        toast.show(`Copied to ${c.id}.`, false, 4000);
      } else {
        await api.dismissUpdate(id);
      }
      await loadOverview();
      await loadList(action !== "done");
    } catch (e) {
      toast.show(errorText(e), true);
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
    const key = sheet || syncSheet ? null : libraryKey(e);
    if (!key) return;
    if (typeof key === "object") {
      const pin = overview?.pins[key.pin - 1];
      if (!pin) return;
      void openPin(pin.fullId);
    } else if (key === "find") {
      if (special) setNav("all");
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
      onEvent("shelf-changed", () => void refresh()),
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
      onaddlib={() => (sheet = "addlib")}
      onsyncproblem={(lib) => (syncSheet = lib)}
    />
    {#if special}
      <ItemList
        {title}
        {groups}
        {selected}
        loading={listing}
        empty={nav === "inbox"
          ? "The Inbox is empty. Drafts that models save, and updates from team libraries, show up here."
          : nav === "proposals"
            ? "No proposals. Edit a prompt in a team library to start one."
            : "Every file reads."}
        onselect={(key) => void select(key)}
      >
        {#snippet tools()}
          {#if nav === "proposals"}
            <button class="btn sm" onclick={() => void refreshProposals()} disabled={refreshing} title="Ask the host for each state">
              <Icon name="sync" small />{refreshing ? "Checking…" : "Refresh"}
            </button>
          {/if}
        {/snippet}
      </ItemList>
    {:else}
      <PromptList
        {title}
        {items}
        selected={selected}
        bind:query
        bind:searchEl
        loading={listing}
        personal={overview.personal}
        onselect={(id) => void select(id)}
        onnew={() => (sheet = "new")}
        onopen={() => detail && (sheet = "send")}
      />
    {/if}
    {#if update}
      <UpdateDetail
        {update}
        onallow={() => void onUpdate("allow")}
        onkeep={() => void onUpdate("keep")}
        oncopy={() => void onUpdate("copy")}
        ondone={() => void onUpdate("done")}
      />
    {:else if problem}
      <ProblemDetail {problem} onfixed={() => void changed(null, "Saved. The file reads now.")} />
    {:else if detail}
      <Detail
        bind:this={detailView}
        bind:editing
        {detail}
        {filler}
        personal={overview.personal}
        onpin={() => void togglePin()}
        onsend={() => (sheet = "send")}
        onsaved={(id) => void saved(id)}
        onaction={(a) => promptActions?.run(a)}
      />
      <PromptActions
        bind:this={promptActions}
        {detail}
        libraries={overview.libraries}
        personal={overview.personal}
        ondone={(next, message) => void changed(next, message)}
        onerror={(m) => toast.show(m, true)}
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
{:else if sheet === "addlib"}
  <AddLibrarySheet
    onadded={(name) => {
      sheet = null;
      void changed(null, `Added ${name}.`).then(() => setNav(`lib:${name}`));
    }}
    oncancel={() => (sheet = null)}
  />
{:else if syncLib?.lastSync?.conflicts.length}
  <ConflictSheet
    library={syncLib.name}
    files={syncLib.lastSync.conflicts}
    ondone={(r) => {
      syncSheet = null;
      synced([r]);
      void refresh();
    }}
    oncancel={() => (syncSheet = null)}
  />
{:else if syncLib?.lastSync?.error}
  <SyncErrorSheet
    library={syncLib.name}
    last={syncLib.lastSync}
    onretry={() => void sync()}
    onsettings={() => {
      syncSheet = null;
      void api.openWindow("settings", "libraries");
    }}
    oncancel={() => (syncSheet = null)}
  />
{/if}

{#if toast.text}<div transition:rise|global class="toast" class:bad={toast.bad} role="status">{toast.text}</div>{/if}

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

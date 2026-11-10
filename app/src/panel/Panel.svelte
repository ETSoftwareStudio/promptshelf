<!-- The quick panel behind the tray icon and the shortcut: pins first, then
     search, a quick fill, and Copy or Paste. -->
<script lang="ts">
  import { rise } from "../lib/motion";
  import { onMount, tick } from "svelte";
  import FilledPrompt from "../components/FilledPrompt.svelte";
  import Icon from "../components/Icon.svelte";
  import { api, copyInBrowser, errorText, inTauri, onEvent } from "../lib/api";
  import { Filler } from "../lib/filler.svelte";
  import { libraryLabel } from "../lib/format";
  import { modKey } from "../lib/hotkey";
  import { onFocus } from "../lib/platform";
  import { Toast } from "../lib/toast.svelte";
  import type { Summary } from "../lib/types";

  let step = $state<"list" | "fill" | "preview">("list");
  let query = $state("");
  let items = $state<Summary[]>([]);
  let hasConfig = $state(true);
  let index = $state(0);
  let current = $state<Summary | null>(null);
  let pasteNote = $state<string | null>(null);
  let busy = $state(false);
  let searchEl = $state<HTMLInputElement>();
  let listEl = $state<HTMLElement>();
  let fillEl = $state<HTMLElement>();
  const filler = new Filler();
  const toast = new Toast();

  const pinned = $derived(query.trim() ? [] : items.filter((s) => s.pin));
  const rest = $derived(query.trim() ? items : items.filter((s) => !s.pin));
  const ordered = $derived([...pinned, ...rest]);

  let seq = 0;
  async function load() {
    const n = ++seq;
    try {
      const ov = await api.overview();
      hasConfig = ov.hasConfig;
      if (!ov.hasConfig) return;
      const out = await api.list({ text: query, view: "all" });
      if (n !== seq) return;
      items = out;
      index = Math.min(index, Math.max(0, out.length - 1));
    } catch (e) {
      toast.show(errorText(e), true);
    }
  }

  async function reset() {
    step = "list";
    query = "";
    index = 0;
    current = null;
    filler.clear();
    await load();
    await tick();
    searchEl?.focus();
  }

  async function open(s: Summary | undefined) {
    if (!s) return;
    current = s;
    step = "fill";
    await filler.load(s.fullId);
    await tick();
    // Focus the first field, so you can type at once.
    fillEl?.querySelector<HTMLElement>(".notes input, .notes textarea, .notes button")?.focus();
  }

  async function send(paste: boolean) {
    if (!current) return;
    busy = true;
    try {
      const r = await api.send(current.fullId, filler.values(), paste);
      let copied = r.copied;
      if (!copied) copied = await copyInBrowser(r.text);
      if (!copied) {
        toast.show(r.message ?? "Could not copy.", true);
        return;
      }
      if (paste && !r.pasted && r.message) {
        // Copied, but the keys could not be pressed: say why, keep the panel.
        toast.show(r.message, true, 6000);
        return;
      }
      await api.hidePanel();
      if (!inTauri) toast.show(r.pasted ? "Pasted." : "Copied.");
      await reset();
    } catch (e) {
      toast.show(errorText(e), true);
    } finally {
      busy = false;
    }
  }

  function close() {
    if (step === "preview") step = "fill";
    else if (step === "fill") void reset();
    else if (query) query = "";
    else void api.hidePanel();
  }

  function onkeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;
    if (e.key === "Escape") {
      e.preventDefault();
      close();
      return;
    }
    if (mod && /^[1-9]$/.test(e.key)) {
      const s = items.filter((x) => x.pin)[Number(e.key) - 1];
      if (s) {
        e.preventDefault();
        void open(s);
      }
      return;
    }
    if (mod && e.key === ",") {
      e.preventDefault();
      void api.openWindow("settings");
      return;
    }
    if (step === "list") {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        index = Math.min(ordered.length - 1, index + 1);
        listEl?.querySelector(".mi.on")?.scrollIntoView({ block: "nearest" });
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        index = Math.max(0, index - 1);
      } else if (e.key === "Enter") {
        e.preventDefault();
        void open(ordered[index]);
      }
    } else if (step === "fill" && e.key === "Enter" && (mod || e.target === document.body)) {
      e.preventDefault();
      step = "preview";
    } else if (step === "preview" && e.key === "Enter") {
      e.preventDefault();
      void send(e.altKey ? false : !pasteNote);
    }
  }

  let timer: ReturnType<typeof setTimeout> | undefined;
  let lastQuery = "";
  $effect(() => {
    const q = query;
    if (q === lastQuery) return;
    lastQuery = q;
    index = 0;
    clearTimeout(timer);
    timer = setTimeout(() => void load(), 70);
  });

  onMount(() => {
    void reset();
    void api.getSettings().then((s) => (pasteNote = s.pasteNote ?? null));
    const stops = [
      onEvent("panel-opened", () => void reset()),
      onFocus(() => {
        if (step === "list") void load();
      }),
    ];
    return () => stops.forEach((s) => s());
  });
</script>

<svelte:window {onkeydown} />

<div class="panel">
  {#if step === "list"}
    <label class="search">
      <Icon name="search" small />
      <input
        bind:this={searchEl}
        bind:value={query}
        placeholder="Search prompts"
        spellcheck="false"
        aria-label="Search prompts"
      />
    </label>
    <div class="items" bind:this={listEl} role="listbox" aria-label="Prompts">
      {#if !hasConfig}
        <p class="none">No library yet. Open the library to create one.</p>
      {/if}
      {#if pinned.length}<div class="sh">Pinned</div>{/if}
      {#each pinned as s, i (s.fullId)}
        <button
          class="mi"
          class:on={index === i}
          role="option"
          aria-selected={index === i}
          onclick={() => void open(s)}
          onmousemove={() => (index = i)}
        >
          <Icon name="pin" small class="p" /><span class="t">{s.title}</span>
          <span class="kbd">{modKey(String(s.pin))}</span>
        </button>
      {/each}
      {#if rest.length}<div class="sh">{query.trim() ? "Matches" : "Prompts"}</div>{/if}
      {#each rest as s, j (s.fullId)}
        {@const i = pinned.length + j}
        <button
          class="mi"
          class:on={index === i}
          role="option"
          aria-selected={index === i}
          onclick={() => void open(s)}
          onmousemove={() => (index = i)}
        >
          <Icon name={s.kind === "snippet" ? "text" : s.kind === "context" ? "person" : "braces"} small />
          <span class="t">{s.title}</span>
          {#if s.library !== "personal"}<span class="ctx">{libraryLabel(s.library)}</span>{/if}
        </button>
      {/each}
      {#if hasConfig && !ordered.length}
        <p class="none">{query ? `Nothing matches “${query}”.` : "No prompts yet."}</p>
      {/if}
    </div>
    <footer class="mft">
      <span>Return fills · Esc closes</span>
      <button class="link" onclick={() => void api.openWindow("library")}>Open Library</button>
    </footer>
  {:else if current}
    <header class="ph">
      <button class="back" onclick={close}>
        <Icon name="back" small />{step === "preview" ? "Edit" : current.pin ? "Pinned" : "Back"}
      </button>
      <span class="t">{step === "preview" ? "Send" : current.title}</span>
      {#if current.pin && step === "fill"}<span class="kbd">{modKey(String(current.pin))}</span>{/if}
    </header>
    {#if step === "fill"}
      <div class="pbody" bind:this={fillEl}>
        {#if filler.error}<div class="banner">{filler.error}</div>{/if}
        <FilledPrompt {filler} title={current.title} compact />
      </div>
      <footer class="pfoot">
        <span class="to">{filler.resolving ? "Looking up values…" : "To the clipboard"}</span>
        <button class="btn primary" onclick={() => (step = "preview")} disabled={!filler.view}
          >Send…</button
        >
      </footer>
      <p class="hint">Tab moves between blanks · {modKey("↩")} opens the preview</p>
    {:else}
      {#if filler.view && !filler.view.complete}
        <p class="openn">
          {filler.view.missing.length} open {filler.view.missing.length === 1 ? "blank" : "blanks"}: the
          model fills them from the chat.
        </p>
      {/if}
      <div class="out">{filler.view?.sendText}</div>
      <footer class="pfoot">
        {#if pasteNote}
          <span class="to">{pasteNote}</span>
          <button class="btn primary" onclick={() => void send(false)} disabled={busy}>Copy</button>
        {:else}
          <span class="to">Return pastes · ⌥Return copies</span>
          <button class="btn" onclick={() => void send(false)} disabled={busy}>Copy</button>
          <button class="btn primary" onclick={() => void send(true)} disabled={busy}>Paste</button>
        {/if}
      </footer>
    {/if}
  {/if}
</div>

{#if toast.text}<div transition:rise|global class="toast" class:bad={toast.bad} role="status">{toast.text}</div>{/if}


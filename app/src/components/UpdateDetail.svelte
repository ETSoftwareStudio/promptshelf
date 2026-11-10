<!-- A team update in the Inbox: what changed, and what you can do. The new
     text is in use already; a new or changed command stays off until you
     allow it. -->
<script lang="ts">
  import { api, errorText } from "../lib/api";
  import { libraryLabel, shortDay } from "../lib/format";
  import type { UpdateView } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    update,
    onallow,
    onkeep,
    oncopy,
    ondone,
  }: {
    update: UpdateView;
    onallow: () => void;
    onkeep: () => void;
    oncopy: () => void;
    ondone: () => void;
  } = $props();

  let diff = $state<string | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    const id = update.fullId;
    diff = null;
    error = null;
    api.updateDiff(id).then(
      (d) => (diff = d),
      (e) => (error = errorText(e)),
    );
  });

  const lines = $derived(
    (diff ?? "")
      .split("\n")
      .filter((l) => !/^(diff --git|index |--- |\+\+\+ |\\ No newline)/.test(l))
      .map((l) => ({
        kind: l.startsWith("@@") ? "hunk" : l.startsWith("+") ? "add" : l.startsWith("-") ? "del" : "ctx",
        text: l.startsWith("@@") ? "" : l.slice(1),
      })),
  );
  const blocked = $derived((update.new_commands ?? []).filter((c) => !update.allowed.includes(c)));
  const what = $derived(
    update.change === "added" ? "added" : update.change === "removed" ? "removed" : "changed",
  );
</script>

<section class="detail">
  <header class="dh" data-tauri-drag-region>
    {#if update.change !== "removed"}
      <button class="btn" onclick={oncopy}>Copy to Personal</button>
    {/if}
    {#if update.change !== "added"}
      <button class="btn" onclick={onkeep}>Keep Previous Version</button>
    {/if}
    <button class="btn primary" onclick={ondone}><Icon name="check" small />Done</button>
  </header>
  <div class="body">
    <h2>{update.title}</h2>
    <div class="meta">
      <code>{update.fullId}</code> · {what} in {libraryLabel(update.library)} · {shortDay(update.date)}
    </div>
    {#if blocked.length}
      <div class="banner" role="alert">
        <Icon name="warn" small />
        <div class="grow">
          <p>
            {blocked.length === 1
              ? "This update adds a command. It stays off until you allow it. Read it first:"
              : "This update adds commands. They stay off until you allow them. Read them first:"}
          </p>
          {#each blocked as c (c)}<p><code>{c}</code></p>{/each}
          <div class="acts"><button class="btn sm" onclick={onallow}>Allow</button></div>
        </div>
      </div>
    {:else if update.allowed.length}
      <p class="note"><Icon name="check" small />You allowed <code>{update.allowed.join(", ")}</code>.</p>
    {/if}
    <p class="note">
      {update.change === "removed"
        ? "The team removed this prompt. Keep Previous Version copies its last text into Personal."
        : "The new text is in use. Keep Previous Version copies the old text into Personal for you."}
    </p>
    {#if error}
      <div class="banner err"><Icon name="warn" small /><span class="grow">{error}</span></div>
    {:else if diff !== null && update.change !== "removed"}
      <div class="diff" aria-label="What changed">
        {#each lines as l, i (i)}
          {#if l.kind === "hunk"}<div class="hunk"></div>{:else}<div class="ln {l.kind}">{l.text || " "}</div>{/if}
        {/each}
      </div>
    {/if}
  </div>
</section>

<style>
  .detail {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    background: var(--ground);
  }
  .dh {
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 7px;
    padding: 0 12px;
    flex: none;
  }
  .body {
    padding: 4px 22px 20px;
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
  h2 {
    margin: 0;
    font-size: 19px;
    font-weight: 650;
  }
  .meta {
    font-size: 12px;
    color: var(--ink2);
    margin: 3px 0 14px;
  }
  .note {
    font-size: 12px;
    color: var(--ink2);
    margin: 0 0 12px;
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .note :global(.i) {
    color: var(--ok);
  }
  .diff {
    font: 13.5px/1.6 var(--font-prompt);
    border-radius: 9px;
    background: var(--group);
    box-shadow: inset 0 0 0 0.5px var(--edge);
    padding: 8px 0;
    user-select: text;
    -webkit-user-select: text;
  }
  .ln {
    padding: 0 12px;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .ln.del {
    color: var(--danger);
    text-decoration: line-through;
    text-decoration-color: color-mix(in oklch, var(--danger) 70%, transparent);
  }
  .ln.add {
    background: var(--add-bg);
  }
  .hunk {
    height: 1px;
    margin: 6px 12px;
    background: var(--rule);
  }
  .hunk:first-child {
    display: none;
  }
</style>

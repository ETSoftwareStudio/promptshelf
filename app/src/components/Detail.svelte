<!-- The right column: one prompt as a sentence with footnotes, or its file in
     the editor. -->
<script lang="ts">
  import { api, errorText } from "../lib/api";
  import type { Filler } from "../lib/filler.svelte";
  import { usedTimes } from "../lib/format";
  import { modKey } from "../lib/hotkey";
  import type { Detail } from "../lib/types";
  import Editor from "./Editor.svelte";
  import FilledPrompt from "./FilledPrompt.svelte";
  import Icon from "./Icon.svelte";

  let {
    detail,
    filler,
    editing = $bindable(false),
    onpin,
    onsend,
    onsaved,
  }: {
    detail: Detail;
    filler: Filler;
    editing?: boolean;
    onpin: () => void;
    onsend: () => void;
    onsaved: (fullId: string) => void;
  } = $props();

  let text = $state("");
  let error = $state<string | null>(null);
  let saving = $state(false);

  export function edit() {
    text = detail.file;
    error = null;
    editing = true;
  }

  async function save() {
    saving = true;
    try {
      const id = await api.saveFile(detail.fullId, text);
      editing = false;
      onsaved(id);
    } catch (e) {
      error = errorText(e);
    } finally {
      saving = false;
    }
  }

  const kindLabel = $derived(detail.kind === "draft" ? "draft in the Inbox" : detail.kind);
</script>

<section class="detail">
  <header class="dh" data-tauri-drag-region>
    <button
      class="btn icon"
      class:pressed={detail.pin}
      title={detail.pin ? `Unpin (pinned as ${modKey(String(detail.pin))})` : "Pin"}
      aria-label={detail.pin ? "Unpin" : "Pin"}
      aria-pressed={Boolean(detail.pin)}
      onclick={onpin}
    >
      <Icon name="pin" />
    </button>
    <button
      class="btn icon"
      class:pressed={editing}
      title="Edit the Markdown file"
      aria-label="Edit"
      aria-pressed={editing}
      onclick={() => (editing ? (editing = false) : edit())}
    >
      <Icon name="edit" />
    </button>
    <button class="btn primary" onclick={onsend} disabled={editing} title="Send ({modKey('↩')})">
      <Icon name="send" />Send
    </button>
  </header>

  <div class="body" class:editing>
    <h2>{detail.title}</h2>
    <div class="meta">
      <code>{detail.fullId}</code> · {kindLabel} · {usedTimes(detail.uses)}
      {#if detail.tags.length}· {detail.tags.join(", ")}{/if}
    </div>
    {#if editing}
      <Editor bind:text path={detail.path} {error} {saving} onsave={save} oncancel={() => (editing = false)} />
    {:else}
      {#if filler.error}<div class="banner" role="alert"><Icon name="warn" small />{filler.error}</div>{/if}
      <FilledPrompt {filler} title={detail.title} />
      {#if filler.resolving}<p class="busy">Looking up values…</p>{/if}
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
  .body.editing {
    display: flex;
    flex-direction: column;
    padding-bottom: 0;
  }
  h2 {
    margin: 0;
    font-size: 19px;
    font-weight: 650;
    letter-spacing: -0.01em;
    user-select: text;
    -webkit-user-select: text;
  }
  .meta {
    font-size: 12px;
    color: var(--ink2);
    margin: 3px 0 14px;
  }
  .meta code {
    font-size: 11px;
    color: var(--ink);
  }
  .busy {
    margin: 10px 0 0;
    font-size: 11.5px;
    color: var(--ink3);
  }
</style>

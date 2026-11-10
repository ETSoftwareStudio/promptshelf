<!-- The right column: one prompt as a sentence with footnotes, or its file in
     the editor. -->
<script lang="ts" module>
  /** What the person asked for in the menu or a banner. */
  export type Action =
    | "rename"
    | "delete"
    | "copy"
    | "move"
    | "propose"
    | "withdraw"
    | "allow"
    | "link";
</script>

<script lang="ts">
  import { api, errorText, openLink } from "../lib/api";
  import type { Filler } from "../lib/filler.svelte";
  import { libraryLabel, proposalLabel, usedTimes } from "../lib/format";
  import { modKey } from "../lib/hotkey";
  import type { Detail } from "../lib/types";
  import Editor from "./Editor.svelte";
  import FilledPrompt from "./FilledPrompt.svelte";
  import Icon from "./Icon.svelte";
  import Menu, { type MenuItem } from "./Menu.svelte";

  let {
    detail,
    filler,
    personal = null,
    editing = $bindable(false),
    onpin,
    onsend,
    onsaved,
    onaction,
  }: {
    detail: Detail;
    filler: Filler;
    /** The Personal library's name. */
    personal?: string | null;
    editing?: boolean;
    onpin: () => void;
    onsend: () => void;
    onsaved: (fullId: string) => void;
    onaction?: (action: Action) => void;
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

  /** The commands of blanks that wait for Run Commands. */
  const waiting = $derived(
    filler
      .waiting()
      .map((name) => filler.base?.missing.find((m) => m.name === name)?.from ?? name),
  );
  const kindLabel = $derived(detail.kind === "draft" ? "draft in the Inbox" : detail.kind);
  const team = $derived(detail.writes === "pull-request");
  const prop = $derived(detail.proposalInfo);
  const label = $derived(prop ? proposalLabel(prop) : null);
  const items = $derived.by(() => {
    const out: MenuItem[] = [];
    const act = (a: Action) => () => onaction?.(a);
    out.push({ label: "Copy Link", run: act("link") });
    if (detail.kind === "draft") out.push({ label: "Move to Library…", run: act("move") });
    if (!team) out.push({ label: "Rename…", run: act("rename") });
    if (detail.library !== personal) out.push({ label: "Copy to Personal", run: act("copy") });
    if (prop) out.push({ label: "Withdraw Proposal…", run: act("withdraw"), danger: true, rule: true });
    if (!team) out.push({ label: "Delete…", run: act("delete"), danger: true, rule: true });
    return out;
  });
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
      title={team ? "Edit your version (a proposal)" : "Edit the Markdown file"}
      aria-label="Edit"
      aria-pressed={editing}
      onclick={() => (editing ? (editing = false) : edit())}
    >
      <Icon name="edit" />
    </button>
    {#if items.length}<Menu {items} label="More actions" />{/if}
    <button class="btn primary" onclick={onsend} disabled={editing} title="Send ({modKey('↩')})">
      <Icon name="send" />Send
    </button>
  </header>

  <div class="body" class:editing>
    <h2>{detail.title}</h2>
    <div class="meta">
      <code>{detail.fullId}</code> · {prop ? "your version" : kindLabel} · {usedTimes(detail.uses)}
      {#if detail.tags.length}· {detail.tags.join(", ")}{/if}
      {#if detail.forkedFrom}· copy of <code>{detail.forkedFrom}</code>{/if}
    </div>
    {#if prop && label}
      <div class="banner" class:info={label.tone !== "warn"}>
        <Icon name="branch" small />
        <div class="grow">
          <p>
            <span class="chip {label.tone ?? ''}">{label.chip}</span>
            {label.text}. Your version is on this computer; the team keeps theirs until the pull
            request is merged.
          </p>
          <div class="acts">
            {#if prop.status === "not-sent" || prop.status === "not-merged"}
              <button class="btn sm primary" onclick={() => onaction?.("propose")}>Send Proposal…</button>
            {:else if prop.unsent}
              <button class="btn sm primary" onclick={() => onaction?.("propose")}>Update Pull Request…</button>
            {/if}
            {#if prop.url}
              <button class="btn sm" onclick={() => openLink(prop.url!)}>
                <Icon name="link" small />{prop.number ? `Open #${prop.number}` : "Open on the Host"}
              </button>
            {/if}
          </div>
        </div>
      </div>
    {:else if team}
      <div class="banner info">
        <Icon name="branch" small />
        <span class="grow">
          {libraryLabel(detail.library)} is a team library. Edit makes a proposal on this computer; the
          team's version stays until your pull request is merged.
        </span>
      </div>
    {/if}
    {#if detail.blocked.length}
      <div class="banner" role="alert">
        <Icon name="warn" small />
        <div class="grow">
          <p>
            This prompt runs {detail.blocked.length === 1 ? "a command" : "commands"} from a library you
            do not trust. {detail.blocked.length === 1 ? "It stays" : "They stay"} off until you allow
            {detail.blocked.length === 1 ? "it" : "them"}:
          </p>
          {#each detail.blocked as c (c)}<p><code>{c}</code></p>{/each}
          <div class="acts"><button class="btn sm" onclick={() => onaction?.("allow")}>Allow</button></div>
        </div>
      </div>
    {/if}
    {#if editing}
      <Editor bind:text path={detail.path} {error} {saving} onsave={save} oncancel={() => (editing = false)} />
    {:else}
      {#if waiting.length}
        <div class="banner info">
          <Icon name="warn" small />
          <div class="grow">
            <p>
              A link opened this prompt, so {waiting.length === 1 ? "its command" : "its commands"} did
              not run:
            </p>
            {#each waiting as c (c)}<p><code>{c}</code></p>{/each}
            <div class="acts">
              <button class="btn sm" onclick={() => void filler.runCommands()}>Run Commands</button>
            </div>
          </div>
        </div>
      {/if}
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

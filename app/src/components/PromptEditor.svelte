<!-- The prompt as rich text: a title, tags, and the text with each blank as a
     chip. Click a chip to set what fills it. Preview shows the fill form;
     File shows the Markdown file, for fields the editor does not show. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { onMount } from "svelte";
  import "../styles/editor.css";
  import { api } from "../lib/api";
  import { type BlankEditor, blankTone, createBlankEditor, describe } from "../lib/blank-editor";
  import { PreviewFill } from "../lib/preview.svelte";
  import {
    blankNames,
    cleanSpec,
    freeName,
    readFields,
    removeBlank,
    renameBlank,
    tokens,
    writeFields,
  } from "../lib/promptdoc";
  import type { VarSpec } from "../lib/types";
  import BlankCard from "./BlankCard.svelte";
  import FilledPrompt from "./FilledPrompt.svelte";
  import Icon from "./Icon.svelte";

  type Mode = "text" | "preview" | "file";

  let {
    title = $bindable(),
    tags = $bindable(),
    vars = $bindable(),
    body = $bindable(),
    base = $bindable(null),
    library,
    meta,
    ontitleenter,
    start = "text",
  }: {
    title: string;
    tags: string[];
    vars: Record<string, VarSpec>;
    body: string;
    /** The file the fields are written into. Null hides the File view. */
    base?: string | null;
    /** The library of the prompt: snippets in it need no library name. */
    library: string;
    /** Shown under the title. */
    meta?: Snippet;
    ontitleenter?: () => void;
    /** The view to open in. File, when the frontmatter does not parse. */
    start?: Mode;
  } = $props();

  // svelte-ignore state_referenced_locally
  let mode = $state<Mode>(start);
  // svelte-ignore state_referenced_locally
  let fileText = $state(start === "file" ? (base ?? "") : "");
  let fileError = $state<string | null>(null);
  let host = $state<HTMLDivElement>();
  let list = $state<HTMLDivElement>();
  let titleEl = $state<HTMLInputElement>();
  let tagText = $state("");
  let card = $state<{ name: string } | null>(null);
  let snippets: string[] = [];
  let ed: BlankEditor | null = null;
  const preview = new PreviewFill();

  const used = $derived(blankNames(body));
  const unused = $derived(Object.keys(vars).filter((n) => !used.includes(n) && cleanSpec(vars[n])));

  export function focusTitle() {
    titleEl?.focus();
  }

  export function focusText() {
    ed?.view.focus();
  }

  /** Reads the File view back into the fields. Returns an error when its
   *  frontmatter does not parse. */
  export function finish(): string | null {
    if (mode !== "file") return null;
    try {
      const f = readFields(fileText);
      title = f.title;
      tags = f.tags;
      vars = f.vars;
      body = f.body;
      base = fileText;
      fileError = null;
      return null;
    } catch (e) {
      fileError = `The frontmatter is not valid YAML: ${e instanceof Error ? e.message : String(e)}`;
      return fileError;
    }
  }

  function setMode(next: Mode) {
    if (next === mode) return;
    if (mode === "file" && finish()) return;
    if (next === "file") fileText = writeFields(base ?? "", { title, tags, vars, body });
    card = null;
    mode = next;
    if (next === "text") requestAnimationFrame(() => ed?.view.focus());
  }

  onMount(() => {
    ed = createBlankEditor(host!, {
      doc: body,
      vars: $state.snapshot(vars),
      placeholder: "Write the prompt here. Type {{ for a blank.",
      snippets: () => snippets,
      onchange: (doc) => (body = doc),
      onblank: (name) => (card = { name }),
      onnew: (name) => (card = { name }),
    });
    api
      .list({ text: "", kind: "snippet", view: "all" })
      .then((all) => (snippets = all.map((s) => (s.library === library ? s.id : s.fullId))))
      .catch(() => {});
    return () => ed?.destroy();
  });

  // The text can change outside CodeMirror: a rename, a removal, the File view.
  $effect(() => {
    if (ed && ed.view.state.doc.toString() !== body) {
      ed.view.dispatch({ changes: { from: 0, to: ed.view.state.doc.length, insert: body } });
    }
  });
  $effect(() => ed?.setVars($state.snapshot(vars)));
  $effect(() => ed?.setActive(card?.name ?? null));
  $effect(() => {
    preview.body = body;
    preview.vars = $state.snapshot(vars);
  });

  function addBlank() {
    const name = freeName([...used, ...Object.keys(vars)]);
    ed?.insert(`{{${name}}}`);
    requestAnimationFrame(() => (card = { name }));
  }

  function anchor(): HTMLElement | null {
    if (!card) return null;
    return ed?.chip(card.name) ?? list?.querySelector<HTMLElement>(`[data-row="${CSS.escape(card.name)}"]`) ?? null;
  }

  function setSpec(name: string, spec: VarSpec) {
    vars = { ...vars, [name]: spec };
  }

  function rename(from: string, to: string) {
    const next: Record<string, VarSpec> = {};
    for (const [n, s] of Object.entries(vars)) next[n === from ? to : n] = s;
    vars = next;
    body = renameBlank(body, from, to);
    card = { name: to };
  }

  function remove(name: string) {
    const rest = { ...vars };
    delete rest[name];
    vars = rest;
    body = removeBlank(body, name);
    card = null;
  }

  function putInText(name: string) {
    mode = "text";
    ed?.insert(`{{${name}}}`);
  }

  function addTag() {
    const t = tagText.trim().replace(/,$/, "").trim();
    if (t && !tags.includes(t)) tags = [...tags, t];
    tagText = "";
  }

  function ontagkey(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      addTag();
    } else if (e.key === "Backspace" && !tagText && tags.length) {
      tags = tags.slice(0, -1);
    }
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="pe"
  onkeydown={(e) => {
    // Esc closes the blank card first; the next Esc reaches the window.
    if (e.key === "Escape" && card && !e.defaultPrevented) {
      e.preventDefault();
      card = null;
    }
  }}
>
  <input
    bind:this={titleEl}
    bind:value={title}
    class="input title"
    placeholder="Untitled"
    aria-label="Title"
    spellcheck="false"
    onkeydown={(e) => {
      if (e.key === "Enter" && !e.metaKey && !e.ctrlKey) {
        e.preventDefault();
        if (ontitleenter) ontitleenter();
        else focusText();
      }
    }}
  />
  {@render meta?.()}
  <div class="tags">
    <span class="lbl">Tags</span>
    {#each tags as t (t)}
      <span class="tag">{t}<button aria-label="Remove tag {t}" onclick={() => (tags = tags.filter((x) => x !== t))}>×</button></span>
    {/each}
    <input
      class="tagin"
      bind:value={tagText}
      placeholder={tags.length ? "" : "Add a tag"}
      aria-label="Add a tag"
      spellcheck="false"
      onkeydown={ontagkey}
      onblur={addTag}
    />
  </div>

  <div class="tools">
    {#if mode === "text"}
      <button class="btn sm" onclick={addBlank}><Icon name="plus" small />Add blank</button>
      <span class="hint">or type <kbd>{"{{"}</kbd></span>
    {/if}
    <div class="seg modes" role="radiogroup" aria-label="View">
      <button role="radio" aria-checked={mode === "text"} class:on={mode === "text"} onclick={() => setMode("text")}>Text</button>
      <button role="radio" aria-checked={mode === "preview"} class:on={mode === "preview"} onclick={() => setMode("preview")}
        >Preview</button
      >
      {#if base !== null}
        <button role="radio" aria-checked={mode === "file"} class:on={mode === "file"} onclick={() => setMode("file")}>File</button>
      {/if}
    </div>
  </div>

  {#if fileError}
    <div class="banner" role="alert"><Icon name="warn" small /><span class="grow">{fileError}</span></div>
  {/if}

  <div class="paper" class:preview={mode === "preview"}>
    <div bind:this={host} class="host" hidden={mode !== "text"}></div>
    {#if mode === "preview"}
      <div class="pv">
        <p class="pvh">This is the fill form people see. Try the values: nothing is saved.</p>
        <FilledPrompt filler={preview} />
      </div>
    {:else if mode === "file"}
      <textarea class="src" bind:value={fileText} spellcheck="false" aria-label="Prompt file"></textarea>
    {/if}
  </div>

  {#if mode === "text"}
    <div class="blank-list" bind:this={list}>
      <span class="lbl">Blanks</span>
      {#each used as name, i (name)}
        <span class="nn {blankTone(vars[name])}">{i + 1}</span>
        <button class="nm" data-row={name} onclick={() => (card = { name })}>{name}</button>
        <span class="desc">{describe(vars[name])}</span>
      {:else}
        <span class="none">No blanks yet. Click Add blank, or type {"{{"} in the text.</span>
      {/each}
      {#each unused as name (name)}
        <span class="nn un">!</span>
        <button class="nm" data-row={name} onclick={() => (card = { name })}>{name}</button>
        <span class="warn">
          Has settings but is not in the text.
          <button class="btn sm" onclick={() => putInText(name)}>Put in Text</button>
          <button class="btn sm" onclick={() => remove(name)}>Remove</button>
        </span>
      {/each}
    </div>
  {/if}
</div>

{#if card}
  {@const name = card.name}
  <BlankCard
    {name}
    spec={vars[name] ?? {}}
    uses={tokens(body).filter((t) => t.kind === "blank" && t.name === name).length}
    taken={(n) => n !== name && (used.includes(n) || n in vars)}
    {anchor}
    onspec={(s) => setSpec(name, s)}
    onrename={(to) => rename(name, to)}
    onremove={() => remove(name)}
    onclose={() => (card = null)}
  />
{/if}

<style>
  .pe {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .title {
    font-size: 19px;
    font-weight: 650;
    letter-spacing: -0.01em;
    padding: 5px 9px;
    border-radius: 8px;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 5px;
    margin: 8px 0 12px;
    font-size: 12px;
    color: var(--ink2);
  }
  .lbl {
    margin-right: 2px;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 1px 4px 1px 8px;
    border-radius: 10px;
    background: var(--seg);
    color: var(--ink);
  }
  .tag button {
    border: 0;
    background: none;
    color: var(--ink3);
    cursor: pointer;
    font-size: 13px;
    line-height: 1;
    padding: 0 2px;
  }
  .tagin {
    flex: 1;
    min-width: 90px;
    border: 0;
    background: none;
    font: 12px var(--font-ui);
    color: var(--ink);
    outline: none;
    padding: 2px 0;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }
  .hint {
    font-size: 11.5px;
    color: var(--ink3);
  }
  kbd {
    padding: 0 4px;
    border-radius: 4px;
    background: var(--seg);
    font: 11px var(--font-mono);
    color: var(--ink2);
  }
  .modes {
    margin-left: auto;
  }
  .banner {
    margin-bottom: 8px;
  }
  .paper {
    flex: 1;
    min-height: 200px;
    display: flex;
    flex-direction: column;
    border-radius: 10px;
    background: var(--group);
    box-shadow: inset 0 0 0 0.5px var(--edge);
  }
  .paper:focus-within {
    box-shadow:
      inset 0 0 0 0.5px var(--edge),
      0 0 0 2px var(--focus);
  }
  .paper.preview,
  .paper.preview:focus-within {
    background: var(--ground);
  }
  .host {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .host[hidden] {
    display: none;
  }
  .pv {
    flex: 1;
    padding: 14px 16px;
    overflow: auto;
  }
  .pvh {
    margin: 0 0 8px;
    font-size: 11px;
    color: var(--ink3);
  }
  .src {
    flex: 1;
    margin: 0;
    padding: 14px 16px;
    border: 0;
    background: none;
    resize: none;
    font: 12.5px/1.6 var(--font-mono);
    color: var(--ink);
    tab-size: 2;
    outline: none;
    user-select: text;
    -webkit-user-select: text;
  }
  .blank-list {
    display: grid;
    grid-template-columns: 14px auto 1fr;
    align-items: center;
    gap: 5px 8px;
    margin-top: 10px;
    font-size: 11.5px;
    color: var(--ink2);
  }
  .blank-list .lbl {
    grid-column: 1 / -1;
    font-size: 11px;
    font-weight: 600;
    color: var(--ink3);
  }
  .blank-list .nn {
    margin-top: 0;
  }
  .nn.un {
    background: none;
    box-shadow: inset 0 0 0 1.5px var(--warn);
    color: var(--warn-ink);
  }
  .nm {
    justify-self: start;
    padding: 0;
    border: 0;
    background: none;
    font: 500 11.5px var(--font-mono);
    color: var(--ink);
    cursor: pointer;
  }
  .nm:hover {
    text-decoration: underline;
  }
  .desc {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .none {
    grid-column: 1 / -1;
    color: var(--ink3);
  }
  .warn {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    color: var(--warn-ink);
  }
</style>

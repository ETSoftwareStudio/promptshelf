<!-- The settings of one blank, next to its chip: its name, its type and what
     the type needs. Every change goes to the prompt at once. -->
<script lang="ts">
  import { ICONS, sourceKey } from "../lib/blank-editor";
  import { type BlankType, blankType, isVarName, MENU_AT } from "../lib/promptdoc";
  import type { VarSpec } from "../lib/types";

  let {
    name,
    spec,
    uses,
    taken,
    anchor,
    onspec,
    onrename,
    onremove,
    onclose,
  }: {
    name: string;
    spec: VarSpec;
    /** How many times the text has this blank. */
    uses: number;
    /** True when another blank has this name. */
    taken: (name: string) => boolean;
    /** The chip to show the card next to. Chips are drawn again when the
     *  text changes, so this looks the chip up each time. */
    anchor: () => HTMLElement | null;
    onspec: (spec: VarSpec) => void;
    onrename: (to: string) => void;
    onremove: () => void;
    onclose: () => void;
  } = $props();

  const TYPES: { key: BlankType; label: string }[] = [
    { key: "text", label: "Text" },
    { key: "choice", label: "Choice" },
    { key: "auto", label: "Automatic" },
  ];
  const SOURCES = [
    { key: "conversation", label: "Conversation", tone: "c", help: "An agent fills it from the chat. In the app you type it." },
    { key: "clipboard", label: "Clipboard", tone: "y", help: "The app pastes what you copied last." },
    { key: "selection", label: "Selection", tone: "y", help: "The text selected in the front app." },
    { key: "branch", label: "Branch", tone: "g", help: "The current git branch name." },
    { key: "command", label: "Command", tone: "g", help: "The app runs this command and uses its output." },
    { key: "last", label: "Last value", tone: "y", help: "The value from the last time you used the prompt." },
  ];

  // The type stays as you picked it, also before a Choice has choices.
  // svelte-ignore state_referenced_locally
  let type = $state<BlankType>(blankType(spec));
  // Settings of the other types, so a switch back gets them again.
  let stash: VarSpec = {};
  let nameText = $state("");
  let nameError = $state<string | null>(null);
  let card = $state<HTMLElement>();
  let pos = $state({ top: 0, left: 0 });
  let dragFrom = $state<number | null>(null);
  let dragOver = $state<number | null>(null);

  $effect.pre(() => {
    nameText = name;
    nameError = null;
  });

  const choices = $derived(spec.choices ?? []);
  const key = $derived(spec.from ? sourceKey(spec.from) : "conversation");
  const source = $derived(SOURCES.find((s) => s.key === key)!);

  function place() {
    const el = anchor();
    if (!el || !card) return;
    const r = el.getBoundingClientRect();
    const w = card.offsetWidth;
    const h = card.offsetHeight;
    const left = Math.max(8, Math.min(r.left, window.innerWidth - w - 8));
    let top = r.bottom + 6;
    if (top + h > window.innerHeight - 8) top = Math.max(8, r.top - h - 6);
    pos = { top, left };
  }
  // Again when the type or the name changes: the card grows or the chip moves.
  $effect(() => {
    void type;
    void name;
    requestAnimationFrame(place);
  });

  function set(patch: Partial<VarSpec>) {
    const next: VarSpec = { ...spec, ...patch };
    for (const k of Object.keys(next) as (keyof VarSpec)[]) {
      if (next[k] === undefined || next[k] === "") delete next[k];
    }
    onspec(next);
  }

  function setType(t: BlankType) {
    if (t === type) return;
    stash = { ...stash, ...spec };
    type = t;
    if (t === "text") set({ from: undefined, choices: undefined, pattern: undefined });
    if (t === "choice") {
      const list = spec.choices ?? stash.choices ?? [];
      set({
        from: undefined,
        pattern: undefined,
        hint: undefined,
        choices: list,
        default: spec.default && list.includes(spec.default) ? spec.default : undefined,
      });
      requestAnimationFrame(() => card?.querySelector<HTMLInputElement>(".chr.add input")?.focus());
    }
    if (t === "auto") set({ choices: undefined, default: undefined, from: stash.from || "conversation" });
  }

  function setSource(k: string) {
    if (k === key) return;
    const keep = spec.from && sourceKey(spec.from) === "command" ? spec.from : stash.from;
    if (spec.from) stash = { ...stash, from: spec.from };
    const from = k === "command" ? (keep && sourceKey(keep) === "command" ? keep : "git diff --staged") : k;
    set({ from, pattern: undefined });
  }

  function commitName() {
    const to = nameText.trim();
    if (to === name) return void (nameError = null);
    if (!isVarName(to)) {
      nameError = "Use letters, digits, _ or -. Start with a letter.";
      return;
    }
    if (taken(to)) {
      nameError = `There is a blank named ${to} already.`;
      return;
    }
    nameError = null;
    onrename(to);
  }

  function setChoices(list: string[], def = spec.default) {
    set({ choices: list, default: def && list.includes(def) ? def : undefined });
  }

  function editChoice(i: number, value: string) {
    const list = [...choices];
    const was = list[i];
    list[i] = value;
    setChoices(list, spec.default === was ? value : spec.default);
  }

  function move(from: number, to: number) {
    if (to < 0 || to >= choices.length || from === to) return;
    const list = [...choices];
    const [c] = list.splice(from, 1);
    list.splice(to, 0, c);
    setChoices(list);
  }

  function focusRow(i: number) {
    requestAnimationFrame(() =>
      card?.querySelectorAll<HTMLInputElement>(".chr[data-i] input")[i]?.focus(),
    );
  }

  function onchoicekey(e: KeyboardEvent, i: number) {
    if (e.altKey && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
      e.preventDefault();
      const to = i + (e.key === "ArrowUp" ? -1 : 1);
      move(i, to);
      focusRow(Math.max(0, Math.min(choices.length - 1, to)));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const list = [...choices];
      list.splice(i + 1, 0, "");
      setChoices(list);
      focusRow(i + 1);
    } else if (e.key === "Backspace" && choices[i] === "" && choices.length > 1) {
      e.preventDefault();
      setChoices(choices.filter((_, j) => j !== i));
      focusRow(Math.max(0, i - 1));
    }
  }

  function onaddkey(e: KeyboardEvent) {
    const input = e.currentTarget as HTMLInputElement;
    if (e.key !== "Enter" || !input.value.trim()) return;
    e.preventDefault();
    setChoices([...choices, input.value.trim()]);
    input.value = "";
  }

  function onwindowdown(e: MouseEvent) {
    const t = e.target as Element;
    if (card?.contains(t) || t.closest?.("[data-blank]") || t.closest?.(".blank-list")) return;
    onclose();
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onclose();
    }
  }
</script>

<svelte:window onmousedown={onwindowdown} onresize={place} />

<div
  bind:this={card}
  class="card"
  role="dialog"
  aria-label="Blank {name}"
  tabindex="-1"
  style:top="{pos.top}px"
  style:left="{pos.left}px"
  {onkeydown}
>
  <label class="row2">
    <span>Name</span>
    <input
      class="input name"
      bind:value={nameText}
      spellcheck="false"
      onblur={commitName}
      onkeydown={(e) => e.key === "Enter" && (e.preventDefault(), commitName())}
    />
  </label>
  {#if nameError}<p class="err">{nameError}</p>{/if}
  <div class="row2">
    <span>Type</span>
    <div class="seg types" role="radiogroup" aria-label="Type">
      {#each TYPES as t (t.key)}
        <button role="radio" aria-checked={type === t.key} class:on={type === t.key} onclick={() => setType(t.key)}>
          {@html ICONS[t.key]}{t.label}
        </button>
      {/each}
    </div>
  </div>

  {#if type === "text"}
    <p class="help">People type the value when they use the prompt.</p>
    <label class="row2">
      <span>Hint</span>
      <input class="input" value={spec.hint ?? ""} placeholder="Shown in the empty field" oninput={(e) => set({ hint: e.currentTarget.value })} />
    </label>
    <label class="row2">
      <span>Default</span>
      <input class="input" value={spec.default ?? ""} placeholder="Empty" oninput={(e) => set({ default: e.currentTarget.value })} />
    </label>
  {:else if type === "choice"}
    <p class="help">
      People pick one value.
      {#if choices.length}
        With {choices.length} {choices.length === 1 ? "choice" : "choices"}, the fill form shows
        {choices.length >= MENU_AT ? "a menu" : "buttons"}.
      {:else}
        Add the values below.
      {/if}
    </p>
    <div class="row2 top">
      <span>Choices</span>
      <div class="chl">
        {#if choices.length}<div class="chh">Default</div>{/if}
        {#each choices as c, i (i)}
          <div
            class="chr"
            data-i={i}
            class:drag={dragFrom === i}
            class:over={dragOver === i && dragFrom !== i}
            role="listitem"
            ondragover={(e) => {
              if (dragFrom === null) return;
              e.preventDefault();
              dragOver = i;
            }}
            ondrop={(e) => {
              e.preventDefault();
              if (dragFrom !== null) move(dragFrom, i);
              dragFrom = dragOver = null;
            }}
          >
            <button
              class="radio"
              role="radio"
              aria-checked={spec.default === c && c !== ""}
              aria-label="Default: {c}"
              title={spec.default === c ? "The default. Click to clear it." : "Make this the default"}
              onclick={() => set({ default: spec.default === c ? undefined : c })}
            ></button>
            <span
              class="grip"
              draggable="true"
              title="Drag to reorder"
              role="button"
              tabindex="-1"
              aria-label="Reorder"
              ondragstart={(e) => {
                dragFrom = i;
                e.dataTransfer?.setData("text/plain", c);
              }}
              ondragend={() => (dragFrom = dragOver = null)}
            >
              <svg width="8" height="12" viewBox="0 0 8 12" aria-hidden="true"
                ><g fill="currentColor"
                  ><circle cx="2" cy="2" r="1" /><circle cx="6" cy="2" r="1" /><circle cx="2" cy="6" r="1" /><circle
                    cx="6"
                    cy="6"
                    r="1"
                  /><circle cx="2" cy="10" r="1" /><circle cx="6" cy="10" r="1" /></g
                ></svg
              >
            </span>
            <input
              class="input"
              value={c}
              aria-label="Choice {i + 1}"
              oninput={(e) => editChoice(i, e.currentTarget.value)}
              onkeydown={(e) => onchoicekey(e, i)}
            />
            <button class="x" aria-label="Remove {c}" onclick={() => setChoices(choices.filter((_, j) => j !== i))}>×</button>
          </div>
        {/each}
        <div class="chr add">
          <span class="plus">+</span>
          <input class="input" placeholder="Add a choice" aria-label="Add a choice" onkeydown={onaddkey} />
        </div>
        {#if choices.length > 1}
          <p class="legend">Drag the dots or press ⌥↑ ⌥↓ to reorder. Click a checked default to clear it.</p>
        {/if}
      </div>
    </div>
  {:else}
    <div class="row2 top">
      <span>From</span>
      <div class="srcs">
        {#each SOURCES.filter((s) => s.key !== "last" || key === "last") as s (s.key)}
          <button class="{s.tone} {s.key === key ? 'on' : ''}" aria-pressed={s.key === key} onclick={() => setSource(s.key)}>
            {@html ICONS[s.key]}{s.label}
          </button>
        {/each}
      </div>
    </div>
    <p class="help">{source.help}</p>
    {#if key === "command"}
      <label class="row2">
        <span>Command</span>
        <input class="input mono" value={spec.from ?? ""} placeholder="git diff --staged" spellcheck="false" oninput={(e) => set({ from: e.currentTarget.value || "git diff --staged" })} />
      </label>
    {:else if key === "branch"}
      <label class="row2">
        <span>Pattern</span>
        <input class="input mono" value={spec.pattern ?? ""} placeholder={"[A-Z]+-\\d+  (optional)"} spellcheck="false" oninput={(e) => set({ pattern: e.currentTarget.value })} />
      </label>
    {:else if key === "conversation"}
      <label class="row2">
        <span>Hint</span>
        <input class="input" value={spec.hint ?? ""} placeholder="Shown in the empty field" oninput={(e) => set({ hint: e.currentTarget.value })} />
      </label>
    {/if}
  {/if}

  <div class="cf">
    {uses === 0 ? "Not in the text" : uses === 1 ? "Used once in this prompt" : `Used ${uses} times in this prompt`}
    <button class="btn sm danger" onclick={onremove}>Remove</button>
  </div>
</div>

<style>
  .card {
    position: fixed;
    z-index: 20;
    width: 340px;
    max-height: calc(100vh - 16px);
    overflow: auto;
    padding: 12px 14px;
    border-radius: 10px;
    background: var(--sheet);
    box-shadow:
      0 0 0 0.5px var(--edge),
      0 12px 32px -8px var(--shadow);
    font-size: 12.5px;
    outline: none;
  }
  .row2 {
    display: grid;
    grid-template-columns: 70px 1fr;
    align-items: center;
    gap: 8px;
    margin-bottom: 9px;
  }
  .row2 > span {
    font-size: 11.5px;
    color: var(--ink3);
  }
  .row2.top {
    align-items: start;
  }
  .row2.top > span {
    padding-top: 4px;
  }
  .name {
    font-family: var(--font-mono);
    font-weight: 500;
  }
  .input.mono {
    font-size: 12px;
  }
  .err {
    margin: -4px 0 8px 78px;
    color: var(--danger);
    font-size: 11px;
  }
  .help {
    margin: -3px 0 10px 78px;
    font-size: 11px;
    color: var(--ink3);
  }
  .types {
    width: 100%;
  }
  .types button {
    flex: 1;
    justify-content: center;
  }
  .chl {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .chh {
    font-size: 10.5px;
    color: var(--ink3);
    padding: 0 2px;
  }
  .chr {
    display: grid;
    grid-template-columns: 18px 14px 1fr 22px;
    align-items: center;
    gap: 6px;
    padding: 1px 2px;
    border-radius: 6px;
  }
  .chr.drag {
    opacity: 0.4;
  }
  .chr.over {
    box-shadow: 0 -2px 0 var(--accent);
  }
  .radio {
    width: 14px;
    height: 14px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: var(--field);
    box-shadow: inset 0 0 0 1px var(--ink3);
    cursor: pointer;
  }
  .radio[aria-checked="true"] {
    box-shadow:
      inset 0 0 0 4px var(--accent),
      inset 0 0 0 7px var(--field);
  }
  .radio:focus-visible {
    outline: 2px solid var(--focus);
  }
  .grip {
    cursor: grab;
    color: var(--ink3);
    display: grid;
    place-items: center;
  }
  .x {
    border: 0;
    background: none;
    color: var(--ink3);
    cursor: pointer;
    font-size: 15px;
    line-height: 1;
    padding: 0;
  }
  .x:hover {
    color: var(--danger);
  }
  .chr.add .plus {
    grid-column: 1 / 3;
    text-align: center;
    color: var(--ink3);
  }
  .chr.add .input {
    background: none;
  }
  .legend {
    margin: 5px 0 0 2px;
    font-size: 11px;
    color: var(--ink3);
  }
  .srcs {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .srcs button {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    border: 0;
    padding: 3px 8px;
    border-radius: 6px;
    font: 12px var(--font-ui);
    cursor: pointer;
    background: var(--seg);
    color: var(--ink2);
  }
  .srcs button.on.c {
    background: var(--c-bg);
    color: var(--c-ink);
    box-shadow: inset 0 0 0 1px var(--c);
  }
  .srcs button.on.g {
    background: var(--g-bg);
    color: var(--g-ink);
    box-shadow: inset 0 0 0 1px var(--g);
  }
  .srcs button.on.y {
    background: var(--y-bg);
    color: var(--y-ink);
    box-shadow: inset 0 0 0 1px var(--y);
  }
  .cf {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-top: 9px;
    margin-top: 3px;
    border-top: 1px solid var(--rule);
    font-size: 11px;
    color: var(--ink3);
  }
  .cf .btn {
    margin-left: auto;
  }
  .btn.danger {
    color: var(--danger);
  }
</style>

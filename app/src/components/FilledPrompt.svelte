<!-- The prompt as a sentence, each value tinted by its source, and one
     numbered footnote per value saying where it came from. Footnotes of open
     blanks are the fill form. -->
<script lang="ts">
  import { annotate, sourceLabel, withoutTitle } from "../lib/footnotes";
  import type { Filler } from "../lib/filler.svelte";

  let {
    filler,
    title = "",
    compact = false,
  }: { filler: Filler; title?: string; compact?: boolean } = $props();

  /** A word joiner: a footnote number never starts a line on its own. */
  const JOIN = "\u2060";
  const view = $derived(filler.view);
  const specs = $derived(view?.specs ?? {});
  const parts = $derived(view ? annotate(withoutTitle(view.pieces, title), specs) : null);

  function multiline(name: string): boolean {
    return filler.current(name).includes("\n");
  }
</script>

{#if parts}
  <p class="prose" class:compact>
    {#each parts.shown as s, i (i)}
      {#if s.type === "text"}{s.text}{:else if s.type === "value"}<span
          class="v {s.tone}"
          title={s.name}>{s.text}</span
        >{JOIN}<sup class={s.tone}>{s.n}</sup>{:else}<span class="v ask {s.tone}" title={s.name}
          >{s.text}</span
        >{JOIN}<sup class={s.tone}>{s.n}</sup>{/if}
    {/each}
  </p>

  {#if parts.notes.length}
    <div class="notes">
      {#each parts.notes as note (note.name)}
        {@const spec = specs[note.name] ?? {}}
        {@const reason = filler.reason(note.name)}
        <span class="nn {note.tone}">{note.n}</span>
        <div class="note">
          {#if filler.editable(note.name)}
            {#if spec.choices?.length}
              <span class="choice" role="group" aria-label={note.name}>
                {#each spec.choices as choice (choice)}
                  <button
                    class="btn sm"
                    class:on={filler.current(note.name) === choice}
                    aria-pressed={filler.current(note.name) === choice}
                    onclick={() =>
                      filler.set(note.name, filler.current(note.name) === choice ? "" : choice)}
                    >{choice}</button
                  >
                {/each}
              </span>
            {:else}
              <label class="field">
                <span class="fname">{note.name}</span>
                {#if multiline(note.name)}
                  <textarea
                    class="input area"
                    rows="3"
                    value={filler.current(note.name)}
                    placeholder={spec.hint ?? ""}
                    oninput={(e) => filler.set(note.name, e.currentTarget.value)}
                  ></textarea>
                {:else}
                  <input
                    class="input"
                    value={filler.current(note.name)}
                    placeholder={spec.hint ?? ""}
                    spellcheck="false"
                    oninput={(e) => filler.set(note.name, e.currentTarget.value)}
                  />
                {/if}
              </label>
            {/if}
            {#if note.open && spec.from === "conversation"}
              <span class="cap">Left open, the model fills it from the chat.</span>
            {:else if note.open && reason}
              <span class="cap warn">{reason}</span>
            {:else if !note.open && note.source && note.source !== "given"}
              <span class="cap">{sourceLabel(note.source, note.detail)}</span>
            {/if}
          {:else}
            <span class="what">
              {#if note.source === "command"}
                <span class="mono">{note.detail}</span>
              {:else if note.source}
                {sourceLabel(note.source, note.detail)}
              {/if}
              <button class="change" onclick={() => filler.set(note.name, note.value ?? "")}
                >Change</button
              >
            </span>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
{/if}

<style>
  .prose.compact {
    font-size: 14.5px;
    line-height: 1.66;
  }
  .note {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .choice {
    display: flex;
    gap: 5px;
    flex-wrap: wrap;
  }
  .field {
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
  }
  .fname {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--ink3);
    flex: none;
    min-width: 5ch;
  }
  .field .input {
    padding: 3px 7px;
    font-size: 12px;
  }
  .field .area {
    resize: vertical;
    font-family: var(--font-mono);
    font-size: 11.5px;
    line-height: 1.45;
  }
  .cap {
    font-size: 11px;
    color: var(--ink3);
  }
  .cap.warn {
    color: var(--warn-ink);
  }
  .what {
    display: flex;
    gap: 8px;
    align-items: baseline;
    min-width: 0;
    padding-top: 1px;
  }
  .what .mono {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }
  .change {
    color: var(--link);
    font-size: 11px;
    font-weight: 500;
    flex: none;
  }
  .change:hover {
    text-decoration: underline;
  }
</style>

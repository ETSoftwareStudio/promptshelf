<!-- The preview before Send: the message exactly as it will arrive. -->
<script lang="ts">
  import { modKey } from "../lib/hotkey";

  let {
    title,
    text,
    open = 0,
    paste = false,
    pasteNote = null,
    busy = false,
    oncopy,
    onpaste,
    oncancel,
  }: {
    title: string;
    text: string;
    /** How many blanks are still open. */
    open?: number;
    /** Offer "Paste into the front app" (the quick panel does). */
    paste?: boolean;
    pasteNote?: string | null;
    busy?: boolean;
    oncopy: () => void;
    onpaste?: () => void;
    oncancel: () => void;
  } = $props();

  let first = $state<HTMLButtonElement>();
  $effect(() => {
    first?.focus();
  });

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      oncancel();
    } else if (e.key === "Enter" && (e.metaKey || e.ctrlKey || e.target === document.body)) {
      e.preventDefault();
      if (paste && !pasteNote && onpaste) onpaste();
      else oncopy();
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="dim" role="presentation" onclick={oncancel}></div>
<div class="sheet" role="dialog" aria-modal="true" aria-label="Send {title}">
  <h3>Send “{title}”</h3>
  <div class="sub">
    {paste ? "Copies the prompt, then pastes it into the app you came from." : "Copies the prompt. Paste it into any chat."}
  </div>
  {#if open > 0}
    <p class="openn">
      {open === 1 ? "1 blank is" : `${open} blanks are`} still open. The prompt asks the model to fill
      {open === 1 ? "it" : "them"} from the chat.
    </p>
  {/if}
  <div class="out">{text}</div>
  {#if paste && pasteNote}<p class="openn">{pasteNote}</p>{/if}
  <div class="sfoot">
    <span class="grow kbd">{modKey("↩")} sends</span>
    <button class="btn" onclick={oncancel}>Cancel</button>
    {#if paste && onpaste && !pasteNote}
      <button class="btn" onclick={oncopy} disabled={busy}>Copy</button>
      <button bind:this={first} class="btn primary" onclick={onpaste} disabled={busy}>Paste</button>
    {:else}
      <button bind:this={first} class="btn primary" onclick={oncopy} disabled={busy}>Copy</button>
    {/if}
  </div>
</div>

<style>
  .openn {
    margin: 10px 0 0;
    font-size: 11.5px;
    color: var(--ink2);
  }
</style>

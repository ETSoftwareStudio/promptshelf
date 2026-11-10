<!-- Rename a prompt: its title, and its ID if you want. A new ID moves the
     file and updates pins and includes. -->
<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    title: oldTitle,
    id: oldId,
    library,
    error = null,
    busy = false,
    onrename,
    oncancel,
  }: {
    title: string;
    id: string;
    library: string;
    error?: string | null;
    busy?: boolean;
    onrename: (title: string, id: string) => void;
    oncancel: () => void;
  } = $props();

  // svelte-ignore state_referenced_locally
  let title = $state(oldTitle);
  // svelte-ignore state_referenced_locally
  let id = $state(oldId);
  let input = $state<HTMLInputElement>();
  $effect(() => {
    input?.select();
  });
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && oncancel()} />

<div class="dim" role="presentation" onclick={() => !busy && oncancel()}></div>
<form
  class="sheet"
  aria-label="Rename {oldTitle}"
  onsubmit={(e) => (e.preventDefault(), onrename(title, id))}
>
  <h3>Rename “{oldTitle}”</h3>
  <label class="field"><span>Title</span><input bind:this={input} class="input" bind:value={title} required /></label>
  <label class="field">
    <span>ID, used by pins, includes and models: <code>{library}/{id}</code></span>
    <input class="input mono" bind:value={id} pattern="[a-z0-9-]+" spellcheck="false" required />
  </label>
  <p class="fine">
    {id === oldId
      ? "The ID stays, so nothing that uses this prompt changes."
      : "A new ID renames the file. Pins and includes in your libraries follow it. Agents and other computers that still use the old ID must use the new one."}
  </p>
  {#if error}<div class="banner err" role="alert"><Icon name="warn" small /><span class="grow">{error}</span></div>{/if}
  <div class="sfoot">
    <button type="button" class="btn" onclick={oncancel} disabled={busy}>Cancel</button>
    <button type="submit" class="btn primary" disabled={busy}>Rename</button>
  </div>
</form>

<style>
  .banner {
    margin: 10px 0 0;
  }
</style>

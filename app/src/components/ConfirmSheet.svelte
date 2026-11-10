<!-- Asks before an action that is hard to undo. Cancel has the focus. -->
<script lang="ts">
  let {
    title,
    message,
    confirm,
    danger = false,
    busy = false,
    onconfirm,
    oncancel,
  }: {
    title: string;
    message: string;
    confirm: string;
    danger?: boolean;
    busy?: boolean;
    onconfirm: () => void;
    oncancel: () => void;
  } = $props();

  let cancel = $state<HTMLButtonElement>();
  $effect(() => {
    cancel?.focus();
  });
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && oncancel()} />

<div class="dim" role="presentation" onclick={oncancel}></div>
<div class="sheet" role="alertdialog" aria-modal="true" aria-label={title}>
  <h3>{title}</h3>
  <p class="fine">{message}</p>
  <div class="sfoot">
    <button bind:this={cancel} class="btn" onclick={oncancel}>Cancel</button>
    <button class="btn primary" class:danger onclick={onconfirm} disabled={busy}>
      {busy ? "Working…" : confirm}
    </button>
  </div>
</div>

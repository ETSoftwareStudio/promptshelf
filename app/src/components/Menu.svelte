<!-- A "More" button with a short menu. Arrow keys move, Esc closes. -->
<script lang="ts" module>
  export interface MenuItem {
    label: string;
    run: () => void;
    danger?: boolean;
    /** A line above this item. */
    rule?: boolean;
  }
</script>

<script lang="ts">
  import Icon from "./Icon.svelte";

  let { items, label = "More" }: { items: MenuItem[]; label?: string } = $props();

  let open = $state(false);
  let box = $state<HTMLElement>();

  $effect(() => {
    if (open) box?.querySelector<HTMLButtonElement>("button")?.focus();
  });

  function pick(item: MenuItem) {
    open = false;
    item.run();
  }

  function onkeydown(e: KeyboardEvent) {
    if (!open) return;
    const buttons = [...(box?.querySelectorAll<HTMLButtonElement>("button") ?? [])];
    const i = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      open = false;
    } else if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      const by = e.key === "ArrowDown" ? 1 : -1;
      buttons[(i + by + buttons.length) % buttons.length]?.focus();
    }
  }
</script>

<svelte:window
  onclick={(e) => open && !(e.target as Element).closest(".more") && (open = false)}
/>

<div class="more">
  <button
    class="btn icon"
    title={label}
    aria-label={label}
    aria-haspopup="menu"
    aria-expanded={open}
    onclick={() => (open = !open)}
  >
    <Icon name="dots" />
  </button>
  {#if open}
    <div class="menu" role="menu" tabindex="-1" bind:this={box} {onkeydown}>
      {#each items as item (item.label)}
        {#if item.rule}<hr />{/if}
        <button role="menuitem" class:danger={item.danger} onclick={() => pick(item)}>
          {item.label}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .more {
    position: relative;
  }
</style>

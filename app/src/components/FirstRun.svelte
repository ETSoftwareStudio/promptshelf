<!-- First run: no config yet. One button makes a Personal library. -->
<script lang="ts">
  import Icon from "./Icon.svelte";

  let { oncreate }: { oncreate: () => void } = $props();
</script>

<main class="first">
  <aside class="side-empty" data-tauri-drag-region></aside>
  <div class="empty" data-tauri-drag-region>
    <Icon name="books" class="glyph" />
    <h4>No prompts yet</h4>
    <p>
      Prompts live in libraries: folders of Markdown files, usually git repos. Start with your
      Personal library; add your team's later with <code>shelf libs add</code>.
    </p>
    <button class="btn primary" onclick={oncreate}>Create a Personal Library</button>
    <p class="fine">It goes in <code>~/Prompts/personal</code>, like <code>shelf init</code>.</p>
  </div>
</main>

<style>
  .first {
    display: grid;
    grid-template-columns: minmax(176px, 200px) 1fr;
    height: 100%;
  }
  .first .empty {
    background: var(--ground);
  }
  .side-empty {
    margin: 8px 0 8px 8px;
    border-radius: 14px;
    background: var(--side);
    box-shadow: inset 0 0 0 0.5px var(--edge);
  }
  :global(html.vibrant) .side-empty {
    background: color-mix(in oklch, var(--side) 55%, transparent);
  }
  .fine {
    margin-top: 10px;
    font-size: 11.5px;
    color: var(--ink3);
  }
</style>

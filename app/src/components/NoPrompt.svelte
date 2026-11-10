<!-- The detail column with nothing selected. -->
<script lang="ts">
  import Icon from "./Icon.svelte";

  let { problems = [] }: { problems?: { path: string; message: string }[] } = $props();
</script>

<section class="none" data-tauri-drag-region>
  <div class="empty">
    <Icon name="doc" class="glyph" />
    <h4>No prompt selected</h4>
    {#if problems.length}
      <p>
        {problems.length === 1 ? "1 file" : `${problems.length} files`} could not be read:
        <span class="mono" title={problems[0].path}>{problems[0].message}</span>
      </p>
    {/if}
  </div>
</section>

<style>
  .none {
    background: var(--ground);
  }
</style>

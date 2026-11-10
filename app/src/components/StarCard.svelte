<!-- The one-time card at the bottom of the sidebar that asks for a star on
     GitHub. The Rust side decides when it shows (src-tauri/src/tips.rs). It
     hides while a sheet is open: every sheet draws a `.dim` backdrop. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorText } from "../lib/api";
  import { row } from "../lib/motion";
  import Icon from "./Icon.svelte";

  let ask = $state(false);
  let sheetOpen = $state(false);

  async function check() {
    try {
      ask = await api.starCard();
    } catch {
      ask = false;
    }
  }

  async function answer(how: "star" | "not-now" | "never") {
    ask = false;
    try {
      await api.answerStar(how);
    } catch (e) {
      console.error(errorText(e));
    }
  }

  onMount(() => {
    void check();
    const seeSheets = () => (sheetOpen = document.querySelector(".dim") !== null);
    const watch = new MutationObserver(seeSheets);
    watch.observe(document.body, { childList: true, subtree: true });
    seeSheets();
    const onFocus = () => void check();
    window.addEventListener("focus", onFocus);
    return () => {
      watch.disconnect();
      window.removeEventListener("focus", onFocus);
    };
  });
</script>

{#if ask && !sheetOpen}
  <section class="star" aria-label="A star on GitHub" transition:row>
    <p>Enjoying PromptShelf? A star on GitHub helps other people find it.</p>
    <button class="btn sm primary" onclick={() => answer("star")}>
      <Icon name="star" small />Star on GitHub
    </button>
    <div class="more">
      <button onclick={() => answer("not-now")}>Not Now</button>
      <button onclick={() => answer("never")}>Don’t Ask Again</button>
    </div>
  </section>
{/if}

<style>
  .star {
    margin: 10px 0 8px;
    padding: 9px 10px 8px;
    border-radius: 9px;
    background: var(--group);
    box-shadow: inset 0 0 0 0.5px var(--edge);
    display: flex;
    flex-direction: column;
    gap: 7px;
    flex: none;
  }
  p {
    margin: 0;
    font-size: 11.5px;
    line-height: 1.4;
    color: var(--ink2);
    text-wrap: pretty;
  }
  .btn {
    justify-content: center;
  }
  .more {
    display: flex;
    justify-content: space-between;
    gap: 6px;
  }
  .more button {
    font-size: 11px;
    color: var(--ink2);
    padding: 1px 2px;
    border-radius: 4px;
  }
  .more button:hover {
    color: var(--ink);
  }
</style>

<!-- A divider between two columns. Drag it to change the width of the column
     on its left, double-click it to reset the width. The arrow keys move it
     too. Like a macOS split view, the pointer shows the one way it can still
     move at a limit. -->
<script lang="ts">
  let {
    left,
    width,
    min,
    max,
    label,
    ondrag,
    onreset,
    onend,
  }: {
    /** The divider's place from the left of the window, as CSS. */
    left: string;
    /** The width of the column on the left now, in px. */
    width: number;
    min: number;
    max: number;
    label: string;
    /** Asks for a new width. The owner applies its limits. */
    ondrag: (want: number) => void;
    onreset: () => void;
    /** A drag or a key move ended: time to save. */
    onend: () => void;
  } = $props();

  let start: { x: number; width: number } | null = null;
  let dragging = $state(false);

  const cursor = $derived(width <= min ? "e-resize" : width >= max ? "w-resize" : "col-resize");

  function down(e: PointerEvent) {
    if (e.button !== 0) return;
    e.preventDefault();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    start = { x: e.clientX, width };
    dragging = true;
  }

  function move(e: PointerEvent) {
    if (!start) return;
    ondrag(start.width + e.clientX - start.x);
  }

  function up() {
    if (!start) return;
    start = null;
    dragging = false;
    onend();
  }

  function key(e: KeyboardEvent) {
    const step = e.shiftKey ? 50 : 10;
    if (e.key === "ArrowLeft") ondrag(width - step);
    else if (e.key === "ArrowRight") ondrag(width + step);
    else if (e.key === "Home") ondrag(min);
    else if (e.key === "End") ondrag(max);
    else return;
    e.preventDefault();
    onend();
  }

  // While you drag, the whole window shows the resize pointer, so it does not
  // flicker when the pointer runs ahead of the divider.
  $effect(() => {
    const root = document.documentElement;
    if (!dragging) return;
    root.style.setProperty("--drag-cursor", cursor);
    root.classList.add("resizing");
    return () => root.classList.remove("resizing");
  });
</script>

<!-- A focusable separator is the ARIA window splitter pattern, an interactive
     widget, but the a11y check sees a plain separator. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="split"
  style:left
  style:cursor
  role="separator"
  aria-orientation="vertical"
  aria-label={label}
  aria-valuenow={Math.round(width)}
  aria-valuemin={min}
  aria-valuemax={max}
  tabindex="0"
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={up}
  ondblclick={onreset}
  onkeydown={key}
></div>

<style>
  .split {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 9px;
    margin-left: -4px;
    z-index: 5;
    touch-action: none;
  }
  .split:focus-visible {
    outline: none;
    background: linear-gradient(var(--focus), var(--focus)) center / 2px 100% no-repeat;
  }
  :global(html.resizing),
  :global(html.resizing *) {
    cursor: var(--drag-cursor) !important;
  }
</style>

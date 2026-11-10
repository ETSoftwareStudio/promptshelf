<!-- One SVG icon set for the whole app, drawn on a 24-unit grid. -->
<script lang="ts" module>
  const PATHS = {
    search: '<circle cx="11" cy="11" r="6.5"/><path d="M16 16l4.5 4.5"/>',
    plus: '<path d="M12 5v14M5 12h14"/>',
    send: '<circle cx="12" cy="12" r="9"/><path d="M12 16.5v-9M8.5 11L12 7.5l3.5 3.5"/>',
    tray: '<path d="M4 13.5l2.5-7.5h11l2.5 7.5V19H4z"/><path d="M4 13.5h4.5a3.5 3.5 0 007 0H20"/>',
    doc: '<path d="M7 3.5h7l4 4V20.5H7z"/><path d="M14 3.5v4h4"/>',
    clock: '<circle cx="12" cy="12" r="8.5"/><path d="M12 7.5V12l3 2"/>',
    chev: '<path d="M7 10l5 5 5-5"/>',
    back: '<path d="M14.5 6l-6 6 6 6"/>',
    braces:
      '<path d="M9 4.5c-2 0-2.5 1-2.5 2.5v2.5c0 1.3-.8 2.5-2.5 2.5 1.7 0 2.5 1.2 2.5 2.5V17c0 1.5.5 2.5 2.5 2.5M15 4.5c2 0 2.5 1 2.5 2.5v2.5c0 1.3.8 2.5 2.5 2.5-1.7 0-2.5 1.2-2.5 2.5V17c0 1.5-.5 2.5-2.5 2.5"/>',
    text: '<path d="M5 6.5h14M5 10.5h14M5 14.5h9M5 18.5h11"/>',
    person: '<circle cx="12" cy="8.5" r="3.5"/><path d="M5 20a7 7 0 0114 0"/>',
    pin: '<path d="M9.5 3h5a.5.5 0 01.5.5V9l3.4 3.7a.8.8 0 01-.6 1.3H6.2a.8.8 0 01-.6-1.3L9 9V3.5a.5.5 0 01.5-.5z" fill="currentColor" stroke="none"/><path d="M12 14v6.5"/>',
    gear: '<circle cx="12" cy="12" r="3"/><path d="M12 3.5v2.3M12 18.2v2.3M3.5 12h2.3M18.2 12h2.3M6 6l1.6 1.6M16.4 16.4L18 18M6 18l1.6-1.6M16.4 7.6L18 6"/>',
    folder: '<path d="M3.5 7V18.5h17V8.5H11.5L9.5 6H4.5a1 1 0 00-1 1z"/>',
    check: '<path d="M5.5 12.5l4 4 9-9"/>',
    warn: '<path d="M12 4l9 15.5H3z"/><path d="M12 10v4M12 16.8v.2"/>',
    books: '<path d="M4.5 5h4v14h-4zM8.5 5h4v14h-4z"/><path d="M14 6.2l3.8-1 3.4 13.5-3.8 1z"/>',
    edit: '<path d="M5 19l1-4L16 5l3 3L9 18z"/><path d="M14 7l3 3"/>',
    sync: '<path d="M19 12a7 7 0 01-12.2 4.7M5 12a7 7 0 0112.2-4.7"/><path d="M17.5 3.5v4h-4M6.5 20.5v-4h4"/>',
    copy: '<rect x="8" y="8" width="11.5" height="11.5" rx="2"/><path d="M5 15.5V6a1.5 1.5 0 011.5-1.5H16"/>',
    close: '<path d="M7 7l10 10M17 7L7 17"/>',
    dots: '<circle cx="6" cy="12" r="1.3" fill="currentColor"/><circle cx="12" cy="12" r="1.3" fill="currentColor"/><circle cx="18" cy="12" r="1.3" fill="currentColor"/>',
    branch:
      '<circle cx="7" cy="5.5" r="2"/><circle cx="7" cy="18.5" r="2"/><circle cx="17" cy="8" r="2"/><path d="M7 7.5v9M17 10c0 4-10 2.5-10 6.5"/>',
    terminal: '<rect x="3.5" y="5" width="17" height="14" rx="2"/><path d="M7.5 10l3 2.5-3 2.5M12.5 15.5h4"/>',
    link: '<path d="M10 14a4 4 0 005.7 0l3-3a4 4 0 00-5.7-5.7l-1 1"/><path d="M14 10a4 4 0 00-5.7 0l-3 3a4 4 0 005.7 5.7l1-1"/>',
    plug: '<path d="M9 3.5v4M15 3.5v4M7 7.5h10v3a5 5 0 01-10 0z"/><path d="M12 15.5v5"/>',
  } as const;
  export type IconName = keyof typeof PATHS;
</script>

<script lang="ts">
  let { name, small = false, class: cls = "" }: { name: IconName; small?: boolean; class?: string } =
    $props();
</script>

<svg class="i {small ? 's' : ''} {cls}" viewBox="0 0 24 24" aria-hidden="true">
  {@html PATHS[name]}
</svg>

<!-- Settings → About: the icon, the version and the project's links. Every
     link comes from the Rust side and opens in the system browser. -->
<script lang="ts">
  import dark from "../assets/promptshelf-icon-dark.svg";
  import light from "../assets/promptshelf-icon-light.svg";
  import Icon, { type IconName } from "../components/Icon.svelte";
  import { api, errorText, openLink } from "../lib/api";
  import type { Toast } from "../lib/toast.svelte";
  import type { AboutView } from "../lib/types";
  import ThirdPartyList from "./ThirdPartyList.svelte";

  let { toast }: { toast: Toast } = $props();

  let view = $state<AboutView | null>(null);
  /** Shows the third-party list in place of the pane. */
  let licences = $state(false);

  async function load() {
    try {
      view = await api.about();
    } catch (e) {
      toast.show(errorText(e), true);
    }
  }
  void load();

  const OS: Record<string, string> = { macos: "macOS", windows: "Windows", linux: "Linux" };

  type Row = { icon: IconName; label: string; sub: string; url?: string; val?: string };
  const rows = $derived<Row[]>(
    view
      ? [
          { icon: "branch", label: "GitHub Repository", sub: "github.com/ETSoftwareStudio/promptshelf", url: view.links.repo },
          { icon: "doc", label: "What’s New", sub: "The release notes on GitHub.", url: view.links.releases },
          {
            icon: "warn",
            label: "Report a Problem…",
            sub: `Opens a new issue with the version, ${OS[view.os] ?? view.os} and ${view.arch} filled in.`,
            url: view.links.issue,
          },
          { icon: "text", label: "Licence", sub: "Free to use, change and share.", url: view.links.licence, val: "MIT" },
          { icon: "books", label: "Third-party Licences", sub: "The open-source code PromptShelf ships." },
          { icon: "star", label: "Star on GitHub", sub: "A star helps other people find PromptShelf.", url: view.links.repo },
        ]
      : [],
  );
</script>

{#if licences}
  <ThirdPartyList onback={() => (licences = false)} />
{:else if view}
  <div class="hero">
    <picture>
      <source srcset={dark} media="(prefers-color-scheme: dark)" />
      <img src={light} alt="" width="72" height="72" />
    </picture>
    <h1>PromptShelf</h1>
    <p class="ver">Version {view.version}</p>
    <p class="by">
      Made by <button class="linkish" onclick={() => openLink(view!.links.site)}>ET Software Studio</button>
    </p>
  </div>

  <div class="grp">
    {#each rows as row (row.label)}
      <button class="fr lk" onclick={() => (row.url ? openLink(row.url) : (licences = true))}>
        <Icon name={row.icon} class="ico" />
        <span class="stack"><span>{row.label}</span><span class="sub2">{row.sub}</span></span>
        <span class="val">{#if row.val}{row.val}{/if}<Icon name={row.url ? "out" : "chevr"} small /></span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 10px 0 6px;
    text-align: center;
  }
  img {
    display: block;
    margin-bottom: 8px;
  }
  h1 {
    margin: 0;
    font: 600 22px/1.2 var(--font-prompt);
    letter-spacing: -0.005em;
  }
  .ver {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--ink2);
    font-variant-numeric: tabular-nums;
    user-select: text;
    -webkit-user-select: text;
  }
  .by {
    margin: 8px 0 0;
    font-size: 11.5px;
    color: var(--ink2);
  }
  .lk {
    width: 100%;
    text-align: left;
  }
  .lk:hover {
    background: color-mix(in oklch, var(--ink) 4%, transparent);
  }
  .grp > .lk:first-child {
    border-radius: 9px 9px 0 0;
  }
  .grp > .lk:last-child {
    border-radius: 0 0 9px 9px;
  }
  .lk :global(.i.s) {
    color: var(--ink3);
  }
</style>

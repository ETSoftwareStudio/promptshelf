<!-- Send a proposal: the first time it opens a pull request, later it updates
     the same one. -->
<script lang="ts">
  import { onMount } from "svelte";
  import { api, errorText } from "../lib/api";
  import { libraryLabel } from "../lib/format";
  import type { ProposalView } from "../lib/types";
  import Icon from "./Icon.svelte";

  let {
    proposal,
    onsent,
    oncancel,
  }: {
    proposal: ProposalView;
    onsent: (p: ProposalView) => void;
    oncancel: () => void;
  } = $props();

  let title = $state("");
  let body = $state("");
  let busy = $state(false);
  let error = $state<string | null>(null);
  const update = $derived(proposal.status !== "not-sent" && proposal.status !== "not-merged");

  onMount(() => {
    void api.proposalText(proposal.fullId).then((t) => {
      title ||= t.title;
      body ||= t.body;
    });
  });

  async function send(e: SubmitEvent) {
    e.preventDefault();
    busy = true;
    error = null;
    try {
      onsent(await api.propose(proposal.fullId, title, body));
    } catch (err) {
      error = errorText(err);
    } finally {
      busy = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && !busy && oncancel()} />

<div class="dim" role="presentation" onclick={() => !busy && oncancel()}></div>
<form class="sheet" aria-label="Propose {proposal.title}" onsubmit={send}>
  <h3>{update ? "Update" : "Propose"} “{proposal.title}”</h3>
  <div class="sub">
    {libraryLabel(proposal.library)} · branch <code>{proposal.branch}</code>
  </div>
  {#if !update}
    <label class="field"><span>Title</span><input class="input" bind:value={title} required /></label>
    <label class="field">
      <span>Description</span>
      <textarea class="input area" bind:value={body} rows="3"></textarea>
    </label>
  {/if}
  <p class="fine">
    {update
      ? `Pushes your new edits to ${proposal.number ? `#${proposal.number}` : "the pull request"}. The team's version stays until it is merged.`
      : "Pushes the branch and opens the pull request: with gh on GitHub, a push option on GitLab, AGit on Gitea, or a link to open it yourself."}
  </p>
  {#if error}<div class="banner err" role="alert"><Icon name="warn" small /><span class="grow">{error}</span></div>{/if}
  <div class="sfoot">
    <button type="button" class="btn" onclick={oncancel} disabled={busy}>Cancel</button>
    <button type="submit" class="btn primary" disabled={busy}>
      {busy ? "Sending…" : update ? "Update Pull Request" : "Open Pull Request"}
    </button>
  </div>
</form>

<style>
  .area {
    resize: vertical;
    min-height: 56px;
    font-family: var(--font-ui);
  }
  .banner {
    margin: 10px 0 0;
  }
</style>

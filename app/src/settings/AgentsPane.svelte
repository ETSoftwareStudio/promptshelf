<!-- Settings → Agents: the Claude Code skill, the shelf command, and shelf mcp
     in Claude Desktop. Each button does one thing and says what it did. -->
<script lang="ts">
  import Icon from "../components/Icon.svelte";
  import { api, errorText } from "../lib/api";
  import type { Toast } from "../lib/toast.svelte";
  import type { AgentsView } from "../lib/types";

  let { toast }: { toast: Toast } = $props();

  let view = $state<AgentsView | null>(null);
  let busy = $state<string | null>(null);
  let note = $state<Record<string, string>>({});

  async function load() {
    try {
      view = await api.agents();
    } catch (e) {
      toast.show(errorText(e), true);
    }
  }
  void load();

  async function run(key: string, work: () => Promise<string>) {
    busy = key;
    try {
      note[key] = await work();
    } catch (e) {
      toast.show(errorText(e), true);
    } finally {
      busy = null;
      await load();
    }
  }

  const skill = () =>
    run("skill", async () => `Installed ${await api.installSkill()}. Claude Code finds it in new sessions.`);
  const command = () =>
    run("command", async () => {
      const r = await api.installCommand();
      return r.profile
        ? `Linked ${r.path}. Added its folder to PATH in ${r.profile}; new terminal windows find shelf.`
        : `Linked ${r.path}. New terminal windows find shelf.`;
    });
  const claude = () =>
    run("claude", async () => {
      const r = await api.addToClaudeDesktop();
      if (!r.changed) return "Claude Desktop has it already.";
      return r.backup
        ? `Added. The old file is in ${r.backup}. Quit and open Claude Desktop again.`
        : "Added. Quit and open Claude Desktop again.";
    });
</script>

{#if view}
  <div class="gh">Models running shelf can</div>
  <div class="grp">
    <div class="fr"><span>Search, read and fill prompts</span><span class="val">Always</span></div>
    <div class="fr">
      <span class="stack"><span>Save drafts</span><span class="sub2">They land in your Inbox for you to review.</span></span>
      <span class="val">Always</span>
    </div>
    <div class="fr">
      <span class="stack"><span>Edit, sync, propose or allow commands</span><span class="sub2">Only you, unless <code>[agents] allow_writes</code> in config.toml lets them.</span></span>
      <span class="val">Off</span>
    </div>
  </div>

  <div class="gh">Connected tools</div>
  <div class="grp">
    <div class="fr col">
      <div class="line">
        <Icon name="terminal" class="ico" />
        <span class="stack">
          <span>The <code>shelf</code> command</span>
          <span class="sub2 mono">{view.command ?? "Not installed"}</span>
        </span>
        <span class="val">
          {#if view.command}<span class="okt"><Icon name="check" small />Installed</span>{/if}
          {#if !view.commandNote}
            <button class="btn sm" onclick={() => void command()} disabled={busy !== null}>
              {busy === "command" ? "Installing…" : view.command ? "Install Again" : "Install"}
            </button>
          {/if}
        </span>
      </div>
      <span class="sub2 pad">{view.commandNote ?? note.command ?? "Agents in a terminal, Claude Code too, run shelf. It is this app, linked as ~/.local/bin/shelf."}</span>
    </div>
    <div class="fr col">
      <div class="line">
        <Icon name="braces" class="ico" />
        <span class="stack">
          <span>Claude Code skill</span>
          <span class="sub2 mono">{view.skillPath}</span>
        </span>
        <span class="val">
          {#if view.skillInstalled && view.skillCurrent}
            <span class="okt"><Icon name="check" small />Installed</span>
          {:else}
            <button class="btn sm" onclick={() => void skill()} disabled={busy !== null}>
              {busy === "skill" ? "Installing…" : view.skillInstalled ? "Update" : "Install"}
            </button>
          {/if}
        </span>
      </div>
      <span class="sub2 pad">{note.skill ?? "Teaches Claude Code to search, fill and save prompts with shelf."}</span>
    </div>
    <div class="fr col">
      <div class="line">
        <Icon name="plug" class="ico" />
        <span class="stack">
          <span>Claude Desktop</span>
          <span class="sub2">Starts <code>shelf mcp</code> when it needs it. No server keeps running.</span>
        </span>
        <span class="val">
          {#if view.claudeCommand && view.claudeCommand === view.command}
            <span class="okt"><Icon name="check" small />Added</span>
          {:else}
            <button class="btn sm" onclick={() => void claude()} disabled={busy !== null || !view.command}>
              {busy === "claude" ? "Adding…" : "Add to Claude Desktop"}
            </button>
          {/if}
        </span>
      </div>
      <span class="sub2 pad">
        {note.claude ?? (view.command ? `Adds an entry to ${view.claudeConfig ?? "its config"}; the rest stays.` : "Install the shelf command first.")}
      </span>
    </div>
  </div>
{/if}

<style>
  .line {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .pad {
    padding-left: 25px;
    white-space: normal;
  }
  .gh:first-child {
    margin-top: 0;
  }
</style>

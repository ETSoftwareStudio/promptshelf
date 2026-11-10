<!-- What the prompt menu and banners ask for: rename, delete, move a draft,
     copy, propose, withdraw and allow. Each one that changes files asks first
     or shows a sheet. -->
<script lang="ts">
  import type { Action } from "../components/Detail.svelte";
  import ConfirmSheet from "../components/ConfirmSheet.svelte";
  import MoveSheet from "../components/MoveSheet.svelte";
  import ProposeSheet from "../components/ProposeSheet.svelte";
  import RenameSheet from "../components/RenameSheet.svelte";
  import { api, errorText } from "../lib/api";
  import { libraryLabel } from "../lib/format";
  import type { Detail, Kind, LibraryInfo, ProposalView } from "../lib/types";

  let {
    detail,
    libraries,
    personal,
    ondone,
    onerror,
  }: {
    detail: Detail;
    libraries: LibraryInfo[];
    personal: string | null | undefined;
    /** After a change: the prompt to show next (null: the first one), and words for a toast. */
    ondone: (next: string | null | undefined, message: string) => void;
    onerror: (message: string) => void;
  } = $props();

  let open = $state<Action | null>(null);
  let busy = $state(false);
  let error = $state<string | null>(null);

  /** Starts an action. Copy and Allow run at once; the rest open a sheet. */
  export function run(action: Action) {
    error = null;
    if (action === "copy") void work(async () => {
      const c = await api.copyPrompt(detail.fullId);
      return [c.id, `Copied to ${c.id}. Change it there for yourself.`];
    });
    else if (action === "allow") void work(async () => {
      const allowed = await api.allowCommands(detail.fullId);
      return [detail.fullId, `Allowed ${allowed.join(", ")} for ${detail.fullId}.`];
    });
    else open = action;
  }

  async function work(job: () => Promise<[string | null | undefined, string]>) {
    busy = true;
    error = null;
    try {
      const [next, message] = await job();
      open = null;
      ondone(next, message);
    } catch (e) {
      if (open) error = errorText(e);
      else onerror(errorText(e));
    } finally {
      busy = false;
    }
  }

  const close = () => !busy && (open = null);

  function rename(title: string, id: string) {
    void work(async () => [await api.renamePrompt(detail.fullId, title, id), "Renamed."]);
  }

  function move(library: string, kind: Kind) {
    void work(async () => {
      const id = await api.movePrompt(detail.fullId, library, kind);
      const team = libraries.find((l) => l.name === library)?.writes === "pull-request";
      return [
        id,
        team
          ? `Moved to ${libraryLabel(library)} as a proposal. Send it when it is ready.`
          : `Moved to ${libraryLabel(library)}.`,
      ];
    });
  }

  function sent(p: ProposalView) {
    open = null;
    ondone(
      detail.fullId,
      p.url ? `Sent ${p.number ? `#${p.number}` : "the proposal"}: ${p.url}` : "Sent.",
    );
  }
</script>

{#if open === "rename"}
  <RenameSheet
    title={detail.title}
    id={detail.id}
    library={detail.library}
    {error}
    {busy}
    onrename={rename}
    oncancel={close}
  />
{:else if open === "delete"}
  <ConfirmSheet
    title="Delete “{detail.title}”?"
    message="The file {detail.path} is deleted, and its pin with it. A library in git keeps the file in its history; sync removes it from your other computers."
    confirm="Delete"
    danger
    {busy}
    onconfirm={() => void work(async () => (await api.deletePrompt(detail.fullId), [null, "Deleted."]))}
    oncancel={close}
  />
{:else if open === "move"}
  <MoveSheet
    title={detail.title}
    {libraries}
    {personal}
    {error}
    {busy}
    onmove={move}
    oncancel={close}
  />
{:else if open === "propose" && detail.proposalInfo}
  <ProposeSheet proposal={detail.proposalInfo} onsent={sent} oncancel={close} />
{:else if open === "withdraw" && detail.proposalInfo}
  <ConfirmSheet
    title="Withdraw your proposal?"
    message={detail.proposalInfo.number
      ? `Closes #${detail.proposalInfo.number} and removes your version of “${detail.title}” from this computer. The team's version stays.`
      : `Removes your version of “${detail.title}” from this computer. The team's version stays.`}
    confirm="Withdraw"
    danger
    {busy}
    onconfirm={() => void work(async () => (await api.withdraw(detail.fullId), [detail.fullId, "Withdrawn."]))}
    oncancel={close}
  />
{/if}

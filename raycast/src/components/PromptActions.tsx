import { Action, ActionPanel, Icon, Keyboard, showToast, Toast, useNavigation } from "@raycast/api";
import { createDeeplink } from "@raycast/utils";

import type { PromptSummary } from "../lib/contract";
import { showFailure } from "../lib/feedback";
import { setPinned } from "../lib/shelf";
import { fillAndDeliver } from "./fill-flow";

export interface PromptActionsProps {
  prompt: PromptSummary;
  /** The app a paste goes to, for the action title. */
  app?: string;
  onChange: () => void;
}

/** The ⌘K actions for a prompt in the list. */
export function PromptActions({ prompt, app, onChange }: PromptActionsProps) {
  const { push } = useNavigation();
  const pinned = prompt.pinned !== undefined;

  async function togglePin() {
    try {
      await setPinned(prompt.id, !pinned);
      await showToast({ style: Toast.Style.Success, title: pinned ? "Unpinned" : "Pinned", message: prompt.title });
      onChange();
    } catch (error) {
      await showFailure(error, pinned ? "Could not unpin" : "Could not pin");
    }
  }

  return (
    <ActionPanel title={prompt.title}>
      <ActionPanel.Section>
        <Action
          title={app ? `Fill and Paste into ${app}` : "Fill and Paste"}
          icon={Icon.Clipboard}
          onAction={() => fillAndDeliver(prompt, "paste", push)}
        />
        <Action
          title="Copy Filled Prompt"
          icon={Icon.CopyClipboard}
          onAction={() => fillAndDeliver(prompt, "copy", push)}
        />
      </ActionPanel.Section>
      <ActionPanel.Section>
        <Action
          title={pinned ? "Unpin" : "Pin"}
          icon={pinned ? Icon.TackDisabled : Icon.Tack}
          shortcut={{ modifiers: ["cmd", "shift"], key: "p" }}
          onAction={togglePin}
        />
        <Action.Open
          title="Open in Editor"
          target={prompt.path}
          icon={Icon.Pencil}
          shortcut={Keyboard.Shortcut.Common.Open}
        />
        <Action.ShowInFinder path={prompt.path} shortcut={{ modifiers: ["cmd", "shift"], key: "f" }} />
        <Action.CreateQuicklink
          title="Create Quicklink"
          quicklink={{ name: prompt.title, link: createDeeplink({ command: "fill", arguments: { id: prompt.id } }) }}
          shortcut={{ modifiers: ["cmd", "shift"], key: "l" }}
        />
        <Action.CopyToClipboard title="Copy Prompt ID" content={prompt.id} shortcut={Keyboard.Shortcut.Common.Copy} />
      </ActionPanel.Section>
      <ActionPanel.Section>
        <Action
          title="Reload Prompts"
          icon={Icon.ArrowClockwise}
          shortcut={Keyboard.Shortcut.Common.Refresh}
          onAction={onChange}
        />
      </ActionPanel.Section>
    </ActionPanel>
  );
}

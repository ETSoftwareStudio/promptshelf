import { Action, ActionPanel, Icon, List, openExtensionPreferences } from "@raycast/api";

import { CARGO_INSTALL_COMMAND, INSTALL_COMMAND } from "../lib/binary";

/** Shown when the shelf binary is missing. */
export function InstallView() {
  return (
    <List>
      <List.EmptyView
        icon={Icon.Download}
        title="Install PromptShelf"
        description={`This extension calls the shelf command. Install it, then open this command again:\n${INSTALL_COMMAND}`}
        actions={
          <ActionPanel>
            <Action.CopyToClipboard title="Copy Install Command" content={INSTALL_COMMAND} />
            <Action.CopyToClipboard title="Copy Cargo Install Command" content={CARGO_INSTALL_COMMAND} />
            <Action title="Open Extension Preferences" icon={Icon.Gear} onAction={openExtensionPreferences} />
          </ActionPanel>
        }
      />
    </List>
  );
}

/** Shown when shelf has no config yet. */
export function InitView() {
  return (
    <List>
      <List.EmptyView
        icon={Icon.Folder}
        title="Set up PromptShelf"
        description="Run shelf init in a terminal to create your Personal library, then open this command again."
        actions={
          <ActionPanel>
            <Action.CopyToClipboard title="Copy Setup Command" content="shelf init" />
          </ActionPanel>
        }
      />
    </List>
  );
}

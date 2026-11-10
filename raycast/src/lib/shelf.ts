// The client bound to the extension's preferences.

import { getPreferenceValues } from "@raycast/api";

import { createClient, type Preferences } from "./client";

export { ShelfCommandError, type FillOptions } from "./client";

export const {
  shelfBinary,
  workingFolder,
  listPrompts,
  searchPrompts,
  showPrompt,
  fillPrompt,
  listLibraries,
  saveDraft,
  setPinned,
} = createClient(() => getPreferenceValues<Preferences>());

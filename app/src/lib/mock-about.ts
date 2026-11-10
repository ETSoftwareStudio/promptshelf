// The browser mock for Settings → About. The links are the real ones; the
// version, OS and architecture are made up.

import type { AboutView } from "./types";

const REPO = "https://github.com/ETSoftwareStudio/promptshelf";

export function callAbout(cmd: string): unknown {
  switch (cmd) {
    case "about":
      return {
        version: "0.1.0",
        os: "macos",
        arch: "aarch64",
        links: {
          repo: REPO,
          releases: `${REPO}/releases`,
          issue: `${REPO}/issues/new?body=${encodeURIComponent("---\nPromptShelf 0.1.0, macos, aarch64\n")}`,
          licence: `${REPO}/blob/main/LICENSE`,
          site: "https://etsoftware.studio",
        },
      } satisfies AboutView;
  }
  return undefined;
}

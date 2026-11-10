// The browser mock for Settings → About and the star card. The links are the
// real ones; the version, OS and architecture are made up. `#star` in the URL
// shows the star card, for the UI walkthrough.

import type { AboutView } from "./types";

const REPO = "https://github.com/ETSoftwareStudio/promptshelf";

let starred = false;

export function callAbout(cmd: string, a: Record<string, unknown>): unknown {
  switch (cmd) {
    case "star_card":
      return location.hash.includes("star") && !starred;
    case "answer_star":
      starred = true;
      if (a.answer === "star") window.open(REPO, "_blank", "noopener");
      return null;
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

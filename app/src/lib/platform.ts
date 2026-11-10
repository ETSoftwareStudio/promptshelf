// Classes on <html> for each system: `mac` for the overlay title bar, and
// `vibrant` where the window is transparent and macOS draws vibrancy.

import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, inTauri } from "./api";
import { isMac } from "./hotkey";

export function setupWindow(opts: { vibrancy: boolean }): void {
  const root = document.documentElement;
  if (isMac) root.classList.add("mac");
  if (inTauri && isMac && opts.vibrancy) root.classList.add("vibrant");
}

/** Calls `cb` each time this window gets focus, after the Tauri side has
 *  forgotten its loaded files, so edits made elsewhere show up. */
export function onFocus(cb: () => void): () => void {
  if (!inTauri) return () => {};
  let stop: (() => void) | undefined;
  let stopped = false;
  void getCurrentWindow()
    .onFocusChanged(({ payload: focused }) => {
      if (focused) void api.reload().then(cb);
    })
    .then((un) => {
      if (stopped) un();
      else stop = un;
    });
  return () => {
    stopped = true;
    stop?.();
  };
}

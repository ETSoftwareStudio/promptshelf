// Calls into the Tauri side. In a plain browser (`vp dev` without Tauri) the
// calls go to a mock, so the UI can be worked on and looked at alone.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  Detail,
  FillView,
  Kind,
  Overview,
  Query,
  SendResult,
  Settings,
  SettingsView,
  Summary,
  SyncReport,
  Value,
} from "./types";

export const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (inTauri) return invoke<T>(cmd, args);
  if (import.meta.env.DEV) {
    const mock = await import("./mock");
    return mock.call(cmd, args ?? {}) as Promise<T>;
  }
  throw new Error("PromptShelf runs inside its desktop app.");
}

export const api = {
  overview: () => call<Overview>("overview"),
  reload: () => call<void>("reload"),
  list: (query: Query) => call<Summary[]>("list_prompts", { query }),
  get: (id: string) => call<Detail>("get_prompt", { id }),
  fill: (id: string, given: Record<string, string> = {}) =>
    call<FillView>("fill_prompt", { id, given }),
  render: (id: string, values: Record<string, Value>) =>
    call<FillView>("render_prompt", { id, values }),
  send: (id: string, values: Record<string, Value>, paste = false) =>
    call<SendResult>("send_prompt", { id, values, paste }),
  pin: (id: string, at?: number) => call<string[]>("pin_prompt", { id, at: at ?? null }),
  unpin: (id: string) => call<string[]>("unpin_prompt", { id }),
  saveFile: (id: string, text: string) => call<string>("save_prompt_file", { id, text }),
  newPrompt: (title: string, kind: Kind, library?: string | null) =>
    call<string>("new_prompt", { title, kind, library: library ?? null }),
  sync: (library?: string | null) =>
    call<SyncReport[]>("sync_libraries", { library: library ?? null }),
  initPersonal: () => call<string>("init_personal"),
  getSettings: () => call<SettingsView>("get_settings"),
  setSettings: (settings: Settings) => call<SettingsView>("set_settings", { settings }),
  openWindow: (name: "library" | "settings" | "panel") => call<void>("open_window", { name }),
  hidePanel: () => call<void>("hide_panel"),
};

/** Listens to an event from the Tauri side. Returns a function that stops. */
export function onEvent<T>(name: string, cb: (payload: T) => void): () => void {
  if (!inTauri) return () => {};
  let stop: (() => void) | undefined;
  let stopped = false;
  void listen<T>(name, (e) => cb(e.payload)).then((un) => {
    if (stopped) un();
    else stop = un;
  });
  return () => {
    stopped = true;
    stop?.();
  };
}

/** Copies text with the browser, when the system tool is missing. */
export async function copyInBrowser(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

/** The error text of a failed command. */
export function errorText(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return String(e);
}

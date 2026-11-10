// Calls into the Tauri side. In a plain browser (`vp dev` without Tauri) the
// calls go to a mock, so the UI can be worked on and looked at alone.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AboutView,
  AgentsView,
  Copied,
  CopiedLink,
  Detail,
  Environment,
  InboxView,
  LibraryStatus,
  Link,
  McpSetup,
  NewLibrary,
  ProposalView,
  Resolution,
  Writes,
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
  /** Runs the resolvers. `run: false` holds shell commands back. */
  fill: (id: string, given: Record<string, string> = {}, run = true) =>
    call<FillView>("fill_prompt", { id, given, run }),
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
  /** First run: a folder for Personal, and a repo to clone into it. */
  initPersonal: (path?: string | null, url?: string | null) =>
    call<string>("init_personal", { path: path ?? null, url: url ?? null }),
  pickFolder: (title: string, start?: string | null) =>
    call<string | null>("pick_folder", { title, start: start ?? null }),
  environment: () => call<Environment>("environment"),
  setGitIdentity: (name: string, email: string) =>
    call<Environment>("set_git_identity", { name, email }),
  libraryStatus: () => call<LibraryStatus[]>("library_status"),
  addLibrary: (library: NewLibrary) => call<string>("add_library", { library }),
  updateLibrary: (name: string, writes?: Writes | null, trusted?: boolean | null) =>
    call<void>("update_library", { name, writes: writes ?? null, trusted: trusted ?? null }),
  removeLibrary: (name: string) => call<string>("remove_library", { name }),
  connectRemote: (name: string, url: string) => call<SyncReport>("connect_remote", { name, url }),
  createGithubRepo: (name: string, repo: string) =>
    call<SyncReport>("create_github_repo", { name, repo }),
  resolveConflict: (library: string, how: Resolution) =>
    call<SyncReport>("resolve_conflict", { library, how }),
  agents: () => call<AgentsView>("agents_status"),
  installSkill: () => call<string>("install_skill"),
  installCommand: () => call<{ path: string; profile?: string | null }>("install_command"),
  addToClaudeDesktop: () => call<McpSetup>("add_to_claude_desktop"),
  deletePrompt: (id: string) => call<void>("delete_prompt", { id }),
  renamePrompt: (id: string, title: string, newId: string) =>
    call<string>("rename_prompt", { id, title, newId }),
  movePrompt: (id: string, library: string, kind: Kind) =>
    call<string>("move_prompt", { id, library, kind }),
  copyPrompt: (id: string) => call<Copied>("copy_prompt", { id }),
  readFile: (path: string) => call<string>("read_file", { path }),
  saveRawFile: (path: string, text: string) => call<void>("save_file", { path, text }),
  proposals: (refresh = false) => call<ProposalView[]>("list_proposals", { refresh }),
  proposalText: (id: string) => call<{ title: string; body: string }>("proposal_text", { id }),
  propose: (id: string, title: string, body: string) =>
    call<ProposalView>("propose", { id, title, body }),
  withdraw: (id: string) => call<void>("withdraw_proposal", { id }),
  inbox: () => call<InboxView>("get_inbox"),
  updateDiff: (id: string) => call<string>("update_diff", { id }),
  allowCommands: (id: string) => call<string[]>("allow_commands", { id }),
  keepPrevious: (id: string) => call<Copied>("keep_previous", { id }),
  dismissUpdate: (id: string) => call<void>("dismiss_update", { id }),
  getSettings: () => call<SettingsView>("get_settings"),
  setSettings: (settings: Settings) => call<SettingsView>("set_settings", { settings }),
  openWindow: (name: "library" | "settings" | "panel", tab?: "libraries" | "agents" | "about") =>
    call<void>("open_window", { name, tab: tab ?? null }),
  hidePanel: () => call<void>("hide_panel"),
  /** The `promptshelf://` link that waits for the library window, once. */
  takeLink: () => call<Link | null>("take_link"),
  /** Tells the log what the library window did with a link. */
  linkHandled: (outcome: string) => call<void>("link_handled", { outcome }),
  copyLink: (id: string) => call<CopiedLink>("copy_link", { id }),
  about: () => call<AboutView>("about"),
  /** True when the sidebar shows the one-time star card. */
  starCard: () => call<boolean>("star_card"),
  answerStar: (answer: "star" | "not-now" | "never") => call<void>("answer_star", { answer }),
};

/** Opens a web page in the system browser. */
export function openLink(url: string): void {
  if (inTauri) void call("open_url", { url });
  else window.open(url, "_blank", "noopener");
}

/** Listens to an event from the Tauri side. Returns a function that stops. */
export function onEvent<T>(name: string, cb: (payload: T) => void): () => void {
  if (!inTauri) {
    // The browser mock sends its events on the window.
    const handler = (e: Event) => cb((e as CustomEvent<T>).detail);
    window.addEventListener(`mock:${name}`, handler);
    return () => window.removeEventListener(`mock:${name}`, handler);
  }
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

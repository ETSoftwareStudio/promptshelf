// The shapes the Tauri commands return. They match `src-tauri/src/service.rs`.

export type Kind = "template" | "snippet" | "context" | "persona" | "workflow" | "draft";

export type Source =
  | "given"
  | "default"
  | "last-used"
  | "branch"
  | "clipboard"
  | "selection"
  | "command"
  | "conversation";

export interface Value {
  value: string;
  source: Source;
  detail?: string;
}

export interface VarSpec {
  from?: string;
  hint?: string;
  default?: string;
  choices?: string[];
  pattern?: string;
}

export interface Filled extends Value {
  name: string;
}

export interface Missing extends VarSpec {
  name: string;
  reason?: string;
}

export type Piece =
  | { type: "text"; text: string }
  | ({ type: "value"; name: string } & Value)
  | { type: "open"; name: string };

export interface FillView {
  fullId: string;
  pieces: Piece[];
  text: string;
  sendText: string;
  complete: boolean;
  values: Filled[];
  missing: Missing[];
  specs: Record<string, VarSpec>;
}

export interface Summary {
  fullId: string;
  id: string;
  library: string;
  kind: Kind;
  title: string;
  description?: string | null;
  tags: string[];
  pin?: number | null;
  uses: number;
  lastUsed?: string | null;
  /** Your proposed version of a team prompt. */
  proposal?: boolean;
}

export interface Detail extends Summary {
  path: string;
  body: string;
  vars: Record<string, VarSpec>;
  forkedFrom?: string | null;
  file: string;
  writes: Writes;
  proposalInfo?: ProposalView | null;
  /** Commands from a library you do not trust that are not allowed yet. */
  blocked: string[];
}

export type Writes = "commit" | "pull-request";

export interface LibraryInfo {
  name: string;
  path: string;
  remote?: string | null;
  writes: Writes;
  personal: boolean;
  trusted: boolean;
  git: boolean;
  exists: boolean;
  count: number;
  lastSync?: LastSync | null;
}

export interface PinInfo {
  fullId: string;
  title: string;
  position: number;
  exists: boolean;
}

export interface Overview {
  configPath: string;
  hasConfig: boolean;
  personal?: string | null;
  libraries: LibraryInfo[];
  kinds: { kind: Kind; count: number }[];
  total: number;
  pins: PinInfo[];
  diagnostics: Diagnostic[];
  /** Team updates in the Inbox. */
  updates: number;
  proposals: number;
  /** Libraries whose last sync had a conflict or an error. */
  syncProblems: string[];
}

export interface Diagnostic {
  path: string;
  message: string;
  library?: string | null;
  /** The path inside the library. */
  file: string;
  help: string;
}

export type View = "all" | "recent";

export interface Query {
  text: string;
  library?: string | null;
  kind?: Kind | null;
  view: View;
}

export interface SyncReport {
  library: string;
  skipped?: string;
  committed?: string;
  pulled: number;
  pushed: number;
  conflicts?: string[];
  updates?: number;
  changes: number;
  error?: string;
  hint?: Hint | null;
}

export interface Hint {
  code: string;
  message: string;
  url?: string;
}

export interface LastSync {
  /** Seconds since 1970. */
  at: number;
  error?: string | null;
  hint?: Hint | null;
  conflicts: string[];
}

export interface LibraryStatus {
  name: string;
  remote?: string | null;
  branch?: string | null;
  ahead: number;
  behind: number;
  changes: number;
  lastSync?: LastSync | null;
}

export interface Environment {
  git: { version?: string | null; name?: string | null; email?: string | null };
  gh: { state: "missing" | "signed-out" | "signed-in"; account?: string };
}

export interface NewLibrary {
  name: string;
  source: string;
  path?: string | null;
  writes?: Writes | null;
  trusted?: boolean | null;
}

export type Resolution = "mine" | "theirs" | "both";

export type ProposalStatus = "not-sent" | "sent" | "changes-requested" | "not-merged" | "gone";

export interface ProposalView {
  fullId: string;
  library: string;
  id: string;
  title: string;
  status: ProposalStatus;
  number?: number | null;
  url?: string | null;
  branch: string;
  /** Edits made after the last send. */
  unsent: boolean;
}

export interface UpdateView {
  library: string;
  id: string;
  fullId: string;
  title: string;
  path: string;
  change: "added" | "edited" | "removed";
  from: string;
  to: string;
  date: string;
  /** Commands this update adds or changes; off until allowed. */
  new_commands?: string[];
  allowed: string[];
}

export interface InboxView {
  drafts: Summary[];
  updates: UpdateView[];
}

export interface Copied {
  id: string;
  path: string;
  forked_from: string;
}

export interface AgentsView {
  skillPath: string;
  skillInstalled: boolean;
  skillCurrent: boolean;
  command?: string | null;
  commandNote?: string | null;
  claudeConfig?: string | null;
  claudeCommand?: string | null;
}

export interface McpSetup {
  path: string;
  backup?: string | null;
  changed: boolean;
}

export interface Settings {
  hotkey: string;
  workDir: string;
}

export interface SettingsView {
  settings: Settings;
  hotkeyError?: string | null;
  pasteNote?: string | null;
  configPath: string;
  settingsPath: string;
}

export interface SendResult {
  text: string;
  copied: boolean;
  pasted: boolean;
  message?: string | null;
}

/** A row of the Inbox, proposals or files-to-fix list. */
export interface Row {
  key: string;
  title: string;
  sub: string;
  /** A chip before the sub line: "Not sent", "#42". */
  chip?: string;
  tone?: "warn" | "ok" | "blue";
  when?: string;
}

export interface Group {
  label: string;
  rows: Row[];
}

/** Where a `promptshelf://` link goes, as `src-tauri/src/links.rs` parses it. */
export type Link =
  | { route: "home" }
  | { route: "open" | "fill"; library: string; id: string }
  | { route: "search"; query: string }
  | { route: "new"; title: string | null }
  | { route: "bad"; reason: string };

export interface CopiedLink {
  link: string;
  copied: boolean;
}

/** Settings → About: the version and the project's links, from the Rust side. */
export interface AboutView {
  version: string;
  /** `std::env::consts::OS`: "macos", "windows", "linux". */
  os: string;
  arch: string;
  links: { repo: string; releases: string; issue: string; licence: string; site: string };
}

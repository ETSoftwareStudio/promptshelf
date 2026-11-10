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
}

export interface Detail extends Summary {
  path: string;
  body: string;
  vars: Record<string, VarSpec>;
  forkedFrom?: string | null;
  file: string;
}

export interface LibraryInfo {
  name: string;
  path: string;
  remote?: string | null;
  writes: "commit" | "pull-request";
  personal: boolean;
  trusted: boolean;
  git: boolean;
  exists: boolean;
  count: number;
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
  diagnostics: { path: string; message: string }[];
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
  changes: number;
  error?: string;
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

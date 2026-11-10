// Types and parsers for the `shelf --json` output (schema 1).
// The CLI is the only interface: this file is the extension's copy of its contract.

export const SCHEMA = 1;

export type Kind = "template" | "snippet" | "context" | "persona" | "workflow" | "draft";

/** Where a value came from. The CLI writes these in kebab-case. */
export type Source =
  "given" | "default" | "last-used" | "branch" | "clipboard" | "selection" | "command" | "conversation";

/** A prompt in `list` and `search`. */
export interface PromptSummary {
  id: string;
  library: string;
  kind: Kind;
  title: string;
  description?: string;
  tags: string[];
  blanks: string[];
  path: string;
  score?: number;
  /** The pin position, 1 to 9. */
  pinned?: number;
}

/** A blank and its spec, from `show` (`prompt.blanks`) and `fill` (`missing`). */
export interface Blank {
  name: string;
  from?: string;
  hint?: string;
  default?: string;
  choices?: string[];
  pattern?: string;
}

export interface PromptDetail {
  id: string;
  library: string;
  kind: Kind;
  title: string;
  description?: string;
  tags: string[];
  body: string;
  path: string;
  blanks: Blank[];
}

export interface FilledValue {
  name: string;
  value: string;
  source: Source;
  detail?: string;
}

export interface MissingBlank extends Blank {
  /** Why the core could not fill it, when a resolver tried and failed. */
  reason?: string;
}

export interface FillResult {
  id: string;
  text: string;
  complete: boolean;
  values: FilledValue[];
  missing: MissingBlank[];
  instruction?: string;
}

export interface Library {
  name: string;
  path: string;
  personal: boolean;
  trusted: boolean;
  prompts: number;
}

export interface SavedPrompt {
  id: string;
  kind: Kind;
  path: string;
}

export interface ShelfErrorBody {
  code: string;
  message: string;
  matches?: string[];
}

/** The JSON did not match the contract. */
export class ContractError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ContractError";
  }
}

type Json = Record<string, unknown>;

function isObject(v: unknown): v is Json {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function str(o: Json, key: string, where: string): string {
  const v = o[key];
  if (typeof v !== "string") throw new ContractError(`${where}: "${key}" is not a string`);
  return v;
}

function optStr(o: Json, key: string): string | undefined {
  const v = o[key];
  return typeof v === "string" ? v : undefined;
}

function strList(o: Json, key: string): string[] {
  const v = o[key];
  return Array.isArray(v) ? v.filter((x): x is string => typeof x === "string") : [];
}

function list(o: Json, key: string, where: string): Json[] {
  const v = o[key];
  if (!Array.isArray(v)) throw new ContractError(`${where}: "${key}" is not a list`);
  return v.filter(isObject);
}

/** Parses stdout and checks the schema version. */
export function parseEnvelope(stdout: string): Json {
  let value: unknown;
  try {
    value = JSON.parse(stdout);
  } catch {
    throw new ContractError("shelf did not print JSON");
  }
  if (!isObject(value)) throw new ContractError("shelf printed JSON that is not an object");
  if (value.schema !== SCHEMA) {
    throw new ContractError(`shelf prints JSON schema ${String(value.schema)}; this extension reads schema ${SCHEMA}`);
  }
  return value;
}

/** The `error` object of a failed command, if the output has one. */
export function parseError(stdout: string): ShelfErrorBody | undefined {
  try {
    const e = parseEnvelope(stdout).error;
    if (!isObject(e)) return undefined;
    return {
      code: typeof e.code === "string" ? e.code : "error",
      message: typeof e.message === "string" ? e.message : "",
      matches: Array.isArray(e.matches) ? strList(e, "matches") : undefined,
    };
  } catch {
    return undefined;
  }
}

function toSummary(o: Json): PromptSummary {
  return {
    id: str(o, "id", "prompt"),
    library: str(o, "library", "prompt"),
    kind: str(o, "kind", "prompt") as Kind,
    title: str(o, "title", "prompt"),
    description: optStr(o, "description"),
    tags: strList(o, "tags"),
    blanks: strList(o, "blanks"),
    path: str(o, "path", "prompt"),
    score: typeof o.score === "number" ? o.score : undefined,
    pinned: typeof o.pinned === "number" ? o.pinned : undefined,
  };
}

function toBlank(o: Json): Blank {
  return {
    name: str(o, "name", "blank"),
    from: optStr(o, "from"),
    hint: optStr(o, "hint"),
    default: optStr(o, "default"),
    choices: Array.isArray(o.choices) ? strList(o, "choices") : undefined,
    pattern: optStr(o, "pattern"),
  };
}

/** `shelf list --json` and `shelf search --json`. */
export function parsePromptList(stdout: string): PromptSummary[] {
  return list(parseEnvelope(stdout), "prompts", "list").map(toSummary);
}

/** `shelf show --json`. */
export function parseShow(stdout: string): PromptDetail {
  const p = parseEnvelope(stdout).prompt;
  if (!isObject(p)) throw new ContractError('show: "prompt" is missing');
  return {
    ...toSummary({ ...p, blanks: [] }),
    body: str(p, "body", "show"),
    blanks: list(p, "blanks", "show").map(toBlank),
  };
}

/** `shelf fill --json`, for exit code 0 (complete) and 3 (blanks open). */
export function parseFill(stdout: string): FillResult {
  const o = parseEnvelope(stdout);
  return {
    id: str(o, "id", "fill"),
    text: str(o, "text", "fill"),
    complete: o.complete === true,
    values: list(o, "values", "fill").map((v) => ({
      name: str(v, "name", "value"),
      value: str(v, "value", "value"),
      source: str(v, "source", "value") as Source,
      detail: optStr(v, "detail"),
    })),
    missing: list(o, "missing", "fill").map((m) => ({ ...toBlank(m), reason: optStr(m, "reason") })),
    instruction: optStr(o, "instruction"),
  };
}

/** `shelf libs list --json`. */
export function parseLibraries(stdout: string): Library[] {
  return list(parseEnvelope(stdout), "libraries", "libs").map((l) => ({
    name: str(l, "name", "library"),
    path: str(l, "path", "library"),
    personal: l.personal === true,
    trusted: l.trusted === true,
    prompts: typeof l.prompts === "number" ? l.prompts : 0,
  }));
}

/** `shelf save --json`. */
export function parseSaved(stdout: string): SavedPrompt {
  const o = parseEnvelope(stdout);
  return { id: str(o, "id", "save"), kind: str(o, "kind", "save") as Kind, path: str(o, "path", "save") };
}

/** Pinned prompts first, by pin position; the rest keep their order. */
export function pinsFirst<T extends { pinned?: number }>(prompts: T[]): { pinned: T[]; rest: T[] } {
  const pinned = prompts.filter((p) => p.pinned !== undefined).sort((a, b) => (a.pinned ?? 0) - (b.pinned ?? 0));
  const rest = prompts.filter((p) => p.pinned === undefined);
  return { pinned, rest };
}

// The calls the extension makes to `shelf`. Each one passes --json.
// No Raycast imports here, so tests can run it against a real binary.

import { homedir } from "node:os";

import { findShelf, candidates, ShelfMissingError } from "./binary";
import {
  type FillResult,
  type Library,
  type PromptDetail,
  type PromptSummary,
  type SavedPrompt,
  type ShelfErrorBody,
  parseError,
  parseFill,
  parseLibraries,
  parsePromptList,
  parseSaved,
  parseShow,
} from "./contract";
import { type Problem, isResult, problemFor } from "./exit";
import { runShelf, type RunOptions, type RunResult } from "./run";

export interface Preferences {
  shelfPath?: string;
  workingFolder?: string;
}

/** A command that ran and failed, with the toast to show. */
export class ShelfCommandError extends Error {
  constructor(
    public readonly exitCode: number,
    public readonly problem: Problem,
    public readonly body?: ShelfErrorBody,
  ) {
    super(problem.message ? `${problem.title}: ${problem.message}` : problem.title);
    this.name = "ShelfCommandError";
  }
}

export interface FillOptions {
  values?: Record<string, string>;
  /** Run the shell commands of `from:`. Off for previews. */
  run?: boolean;
}

/** The client, with the preferences it reads on each call. */
export function createClient(preferences: () => Preferences) {
  /** The path of `shelf`. Throws ShelfMissingError when there is none. */
  function shelfBinary(): string {
    const pref = preferences().shelfPath;
    const bin = findShelf(pref, homedir());
    if (!bin) throw new ShelfMissingError(candidates(pref, homedir()));
    return bin;
  }

  /** The folder where `branch` and shell blanks resolve. */
  function workingFolder(): string {
    return preferences().workingFolder?.trim() || homedir();
  }

  async function call(args: string[], options: RunOptions & { userAction?: boolean } = {}): Promise<RunResult> {
    // Global flags go first: an argument after `--` is never read as a flag.
    const full = ["--json"];
    // Only writes a person asked for in the UI pass --user-action. AI tools never do.
    if (options.userAction) full.push("--user-action");
    full.push(...args);
    const result = await runShelf(shelfBinary(), full, options);
    if (!isResult(result.code)) {
      const body = parseError(result.stdout);
      throw new ShelfCommandError(result.code, problemFor(result.code, body, result.stderr), body);
    }
    return result;
  }

  function libArgs(library?: string): string[] {
    return library ? ["--lib", library] : [];
  }

  async function listPrompts(library?: string): Promise<PromptSummary[]> {
    return parsePromptList((await call(["list", ...libArgs(library)])).stdout);
  }

  async function searchPrompts(query: string, library?: string): Promise<PromptSummary[]> {
    const words = query.trim().split(/\s+/).filter(Boolean);
    if (words.length === 0) return listPrompts(library);
    // `--` keeps a word that starts with "-" from being read as a flag.
    return parsePromptList((await call(["search", ...libArgs(library), "--", ...words])).stdout);
  }

  async function showPrompt(id: string): Promise<PromptDetail> {
    return parseShow((await call(["show", "--", id])).stdout);
  }

  /** Fills a prompt. Resolves for exit 0 (complete) and 3 (blanks open). */
  async function fillPrompt(id: string, options: FillOptions = {}): Promise<FillResult> {
    const args = ["fill", "-C", workingFolder()];
    for (const [name, value] of Object.entries(options.values ?? {})) args.push("--var", `${name}=${value}`);
    if (options.run === false) args.push("--no-run");
    args.push("--", id);
    return parseFill((await call(args)).stdout);
  }

  async function listLibraries(): Promise<Library[]> {
    return parseLibraries((await call(["libs", "list"])).stdout);
  }

  /** Saves text as a draft in the Personal library's Inbox. */
  async function saveDraft(title: string, text: string): Promise<SavedPrompt> {
    return parseSaved((await call(["save", `--title=${title}`], { input: text })).stdout);
  }

  async function setPinned(id: string, pinned: boolean): Promise<void> {
    await call([pinned ? "pin" : "unpin", "--", id], { userAction: true });
  }

  return {
    shelfBinary,
    workingFolder,
    listPrompts,
    searchPrompts,
    showPrompt,
    fillPrompt,
    listLibraries,
    saveDraft,
    setPinned,
  };
}

export type Client = ReturnType<typeof createClient>;

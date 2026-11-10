// Made-up prompts for the browser mock. Every name is made up.

import type { Kind, Value, VarSpec } from "./types";

export interface MockPrompt {
  library: string;
  id: string;
  kind: Kind;
  title: string;
  description?: string;
  tags: string[];
  vars: Record<string, VarSpec>;
  body: string;
  uses: number;
  lastUsed?: string;
}

export const PROMPTS: MockPrompt[] = [
  {
    library: "personal",
    id: "strict-pr-review",
    kind: "template",
    title: "Strict PR review",
    description: "Blockers only, with a fix for each",
    tags: ["review", "git"],
    vars: {
      focus: { from: "conversation", hint: "what worries you" },
      findings: { choices: ["blockers only", "everything"], hint: "which findings?" },
      diff: { from: "git diff --staged --shortstat" },
    },
    body: "Review this diff like a strict senior reviewer. Focus on {{focus}}. Report {{findings}} and skip style nits. The diff is {{diff}}.\n",
    uses: 23,
    lastUsed: "2026-11-09",
  },
  {
    library: "personal",
    id: "explain-like-a-colleague",
    kind: "template",
    title: "Explain like a colleague",
    description: "Skip the basics and go deep",
    tags: ["explain"],
    vars: { topic: { from: "clipboard" }, audience: { default: "a senior engineer" } },
    body: "Explain {{topic}} to {{audience}}. Skip the basics, name the trade-offs, and say what you would check first.\n",
    uses: 9,
    lastUsed: "2026-10-28",
  },
  {
    library: "personal",
    id: "release-notes",
    kind: "template",
    title: "Release notes from commits",
    description: "Grouped for users, not developers",
    tags: ["git", "release"],
    vars: { log: { from: "git log --oneline -20" }, version: { hint: "the version" } },
    body: "Write release notes for version {{version}} from these commits:\n\n{{log}}\n\nGroup them for users, not developers.\n",
    uses: 4,
    lastUsed: "2026-10-27",
  },
  {
    library: "acme-eng",
    id: "coroutine-audit",
    kind: "template",
    title: "Coroutine audit",
    description: "Shared in Acme eng",
    tags: ["kotlin"],
    vars: { ticket: { from: "branch", pattern: "[A-Z]+-\\d+" } },
    body: "Audit the code for {{ticket}} for coroutine misuse: blocking calls on the main thread, scopes that outlive their owner, missing cancellation checks. List each finding with the line and a fix.\n",
    uses: 2,
    lastUsed: "2026-10-21",
  },
  {
    library: "personal",
    id: "house-rules",
    kind: "snippet",
    title: "House rules",
    description: "Tone and format for every answer",
    tags: [],
    vars: {},
    body: "Answer in short sentences. Use lists for steps. Say when you are not sure.\n",
    uses: 0,
  },
  {
    library: "personal",
    id: "about-me",
    kind: "context",
    title: "About me",
    tags: [],
    vars: {},
    body: "I build desktop and mobile apps, mostly in Rust and Kotlin.\n",
    uses: 0,
  },
  {
    library: "personal",
    id: "rough-idea",
    kind: "draft",
    title: "Rough idea",
    description: "Saved by a model",
    tags: [],
    vars: {},
    body: "Turn this chat into a prompt with blanks for the parts that change.\n",
    uses: 0,
  },
  {
    library: "acme-eng",
    id: "commit-message",
    kind: "template",
    title: "Commit message, conventional",
    description: "Scope from the changed folder",
    tags: ["git"],
    vars: { diff: { from: "git diff --staged" } },
    body: "Write a Conventional Commits message for this change:\n\n{{diff}}\n",
    uses: 0,
  },
];

export const PINS = [
  "personal/strict-pr-review",
  "personal/explain-like-a-colleague",
  "acme-eng/coroutine-audit",
];

export const RESOLVED: Record<string, Value> = {
  diff: {
    value: "+214 −37 across 6 files",
    source: "command",
    detail: "git diff --staged --shortstat",
  },
  topic: { value: "structured concurrency", source: "clipboard" },
  log: {
    value: "a1b2c3d Retry payments once\nd4e5f6a Fix the backoff\n0a1b2c3 Add a timeout",
    source: "command",
    detail: "git log --oneline -20",
  },
  ticket: { value: "PAY-142", source: "branch" },
};

// A pretend shelf for `vp dev` in a plain browser, so the UI can be built and
// looked at without Tauri. Only dev builds load it. Every name is made up.

import { PINS, PROMPTS, RESOLVED, type MockPrompt } from "./mock-data";
import type {
  Detail,
  FillView,
  Kind,
  Overview,
  Piece,
  Query,
  SendResult,
  SettingsView,
  Summary,
  SyncReport,
  Value,
} from "./types";

let pins = [...PINS];

const fullId = (p: MockPrompt) => `${p.library}/${p.id}`;
const find = (id: string) => PROMPTS.find((p) => fullId(p) === id || p.id === id);

function summary(p: MockPrompt): Summary {
  const pos = pins.indexOf(fullId(p));
  return {
    fullId: fullId(p),
    id: p.id,
    library: p.library,
    kind: p.kind,
    title: p.title,
    description: p.description,
    tags: p.tags,
    pin: pos >= 0 ? pos + 1 : null,
    uses: p.uses,
    lastUsed: p.lastUsed,
  };
}

function file(p: MockPrompt): string {
  const lines = ["---", `id: ${p.id}`, `title: ${p.title}`, `kind: ${p.kind}`];
  if (p.description) lines.push(`description: ${p.description}`);
  const vars = Object.entries(p.vars);
  if (vars.length) {
    lines.push("vars:");
    for (const [name, spec] of vars) lines.push(`  ${name}: ${JSON.stringify(spec)}`);
  }
  lines.push("---");
  return `${lines.join("\n")}\n${p.body}`;
}

function render(p: MockPrompt, values: Record<string, Value>): FillView {
  const pieces: Piece[] = [];
  const re = /\{\{\s*([A-Za-z_][\w-]*)\s*\}\}/g;
  let last = 0;
  for (let m = re.exec(p.body); m; m = re.exec(p.body)) {
    if (m.index > last) pieces.push({ type: "text", text: p.body.slice(last, m.index) });
    const v = values[m[1]];
    pieces.push(v?.value ? { type: "value", name: m[1], ...v } : { type: "open", name: m[1] });
    last = m.index + m[0].length;
  }
  if (last < p.body.length) pieces.push({ type: "text", text: p.body.slice(last) });
  const text = pieces
    .map((x) => (x.type === "text" ? x.text : x.type === "value" ? x.value : `{{${x.name}}}`))
    .join("");
  const names = Object.keys(p.vars);
  const missing = names.filter((n) => !values[n]?.value).map((name) => ({ name, ...p.vars[name] }));
  const complete = missing.length === 0;
  return {
    fullId: fullId(p),
    pieces,
    text,
    sendText: complete
      ? text
      : `${text.trimEnd()}\n\nSome blanks in this prompt are not filled. Fill each blank from the conversation so far.\n`,
    complete,
    values: names.filter((n) => values[n]?.value).map((name) => ({ name, ...values[name] })),
    missing,
    specs: p.vars,
  };
}

function list(q: Query): Summary[] {
  let all = PROMPTS.filter(
    (p) => (!q.library || p.library === q.library) && (!q.kind || p.kind === q.kind),
  );
  const words = q.text.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length) {
    all = all.filter((p) =>
      words.every((w) =>
        `${p.id} ${p.title} ${p.tags.join(" ")} ${p.body}`.toLowerCase().includes(w),
      ),
    );
    return all.map(summary);
  }
  if (q.view === "recent") {
    return all
      .filter((p) => p.uses > 0)
      .sort((a, b) => (b.lastUsed ?? "").localeCompare(a.lastUsed ?? ""))
      .map(summary);
  }
  const s = all.map(summary);
  return [
    ...s.filter((x) => x.pin).sort((a, b) => (a.pin ?? 0) - (b.pin ?? 0)),
    ...s.filter((x) => !x.pin),
  ];
}

function overview(): Overview {
  const kinds = new Map<Kind, number>();
  for (const p of PROMPTS) kinds.set(p.kind, (kinds.get(p.kind) ?? 0) + 1);
  return {
    configPath: "~/.config/promptshelf/config.toml",
    hasConfig: !location.hash.includes("firstrun"),
    personal: "personal",
    libraries: [
      {
        name: "personal",
        path: "~/Prompts/personal",
        remote: "git@example.com:you/prompts.git",
        writes: "commit",
        personal: true,
        trusted: true,
        git: true,
        exists: true,
        count: PROMPTS.filter((p) => p.library === "personal").length,
      },
      {
        name: "acme-eng",
        path: "~/Prompts/acme-eng",
        remote: "git@example.com:acme/prompts.git",
        writes: "pull-request",
        personal: false,
        trusted: false,
        git: true,
        exists: true,
        count: PROMPTS.filter((p) => p.library === "acme-eng").length,
      },
    ],
    kinds: [...kinds].map(([kind, count]) => ({ kind, count })),
    total: PROMPTS.length,
    pins: pins.map((id, i) => ({
      fullId: id,
      title: find(id)?.title ?? id,
      position: i + 1,
      exists: Boolean(find(id)),
    })),
    diagnostics: [],
    updates: 0,
    proposals: 0,
    syncProblems: [],
  };
}

const wait = (ms: number) => new Promise((r) => setTimeout(r, ms));

let settings = { hotkey: "CommandOrControl+Shift+Space", workDir: "~/code/payments" };

export async function call(cmd: string, a: Record<string, unknown>): Promise<unknown> {
  const id = a.id as string;
  const p = id ? find(id) : undefined;
  if (id && !p && cmd !== "unpin_prompt") throw new Error(`no prompt with the ID "${id}"`);
  switch (cmd) {
    case "overview":
      return overview();
    case "reload":
      return undefined;
    case "list_prompts":
      return list(a.query as Query);
    case "get_prompt":
      return {
        ...summary(p!),
        path: `~/Prompts/${p!.library}/templates/${p!.id}.md`,
        body: p!.body,
        vars: p!.vars,
        file: file(p!),
        writes: p!.library === "acme-eng" ? "pull-request" : "commit",
        blocked: [],
      } satisfies Detail;
    case "fill_prompt": {
      await wait(250);
      const given = a.given as Record<string, string>;
      const values: Record<string, Value> = {};
      for (const [name, spec] of Object.entries(p!.vars)) {
        if (given[name]) values[name] = { value: given[name], source: "given" };
        else if (RESOLVED[name] && spec.from) values[name] = RESOLVED[name];
        else if (spec.default) values[name] = { value: spec.default, source: "default" };
      }
      const view = render(p!, values);
      if (p!.id === "release-notes")
        view.missing = view.missing.map((m) =>
          m.name === "version" ? { ...m, reason: "no tag yet; type the version" } : m,
        );
      return view;
    }
    case "render_prompt":
      return render(p!, a.values as Record<string, Value>);
    case "send_prompt": {
      const view = render(p!, a.values as Record<string, Value>);
      p!.uses += 1;
      return {
        text: view.sendText,
        copied: true,
        pasted: Boolean(a.paste),
        message: null,
      } satisfies SendResult;
    }
    case "pin_prompt":
      pins = [...pins.filter((x) => x !== fullId(p!)), fullId(p!)];
      return pins;
    case "unpin_prompt":
      pins = pins.filter((x) => x !== id && x !== (p ? fullId(p) : ""));
      return pins;
    case "save_prompt_file": {
      const text = a.text as string;
      const body = text.replace(/^---\n[\s\S]*?\n---\n/, "");
      p!.body = body;
      return fullId(p!);
    }
    case "new_prompt": {
      const title = (a.title as string) || "Untitled";
      const np: MockPrompt = {
        library: (a.library as string) ?? "personal",
        id: title
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, "-")
          .replace(/^-|-$/g, ""),
        kind: (a.kind as Kind) ?? "template",
        title,
        tags: [],
        vars: {},
        body: "Write the prompt here. Use {{name}} for a blank.\n",
        uses: 0,
      };
      PROMPTS.push(np);
      return fullId(np);
    }
    case "sync_libraries":
      await wait(600);
      return [
        {
          library: "personal",
          pulled: 1,
          pushed: 2,
          changes: 0,
          committed: "Edit strict-pr-review",
        },
        { library: "acme-eng", pulled: 0, pushed: 0, changes: 0 },
      ] satisfies SyncReport[];
    case "init_personal":
      return "~/Prompts/personal";
    case "get_settings":
    case "set_settings":
      if (cmd === "set_settings") settings = a.settings as typeof settings;
      return {
        settings,
        hotkeyError: null,
        pasteNote: null,
        configPath: "~/.config/promptshelf/config.toml",
        settingsPath: "~/.config/promptshelf/desktop.toml",
      } satisfies SettingsView;
    case "open_window":
    case "hide_panel":
      return undefined;
  }
  throw new Error(`mock: no command ${cmd}`);
}

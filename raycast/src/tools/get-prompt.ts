import { showPrompt } from "../lib/shelf";

type Input = {
  /** The prompt ID from search-prompts, such as "personal/strict-pr-review". A bare ID works when it is unique. */
  id: string;
};

/** Reads one prompt: its template text and its blanks with their hints, sources and choices. */
export default async function tool(input: Input) {
  const p = await showPrompt(input.id);
  return {
    id: p.id,
    title: p.title,
    kind: p.kind,
    library: p.library,
    description: p.description,
    tags: p.tags,
    template: p.body,
    blanks: p.blanks,
  };
}

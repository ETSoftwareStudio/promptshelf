import { searchPrompts } from "../lib/shelf";

type Input = {
  /** Words to look for in the prompt IDs, titles, tags and text. Leave it empty to list every prompt. */
  query?: string;
  /** Only prompts in this library, for example "personal". */
  library?: string;
};

/** Finds prompts in the user's PromptShelf libraries. Pinned prompts have a `pinned` position. */
export default async function tool(input: Input) {
  const prompts = await searchPrompts(input.query ?? "", input.library?.trim() || undefined);
  return prompts.map((p) => ({
    id: p.id,
    title: p.title,
    kind: p.kind,
    library: p.library,
    description: p.description,
    tags: p.tags,
    blanks: p.blanks,
    pinned: p.pinned,
  }));
}

import { fillPrompt } from "../lib/shelf";

type Input = {
  /** The prompt ID from search-prompts, such as "personal/strict-pr-review". */
  id: string;
  /** Values for blanks, one per blank. Fill blanks with `from: conversation` from the chat. */
  values?: {
    /** The blank name. */
    name: string;
    /** The value. For a blank with choices, one of the choices. */
    value: string;
  }[];
};

/**
 * Fills a prompt's blanks. When `complete` is true, `text` is the prompt to follow.
 * When it is false, `missing` lists the open blanks: fill them from the conversation,
 * ask the user only about values you can not work out, and call this tool again.
 */
export default async function tool(input: Input) {
  const values = Object.fromEntries((input.values ?? []).map((v) => [v.name, v.value]));
  const r = await fillPrompt(input.id, { values });
  return {
    id: r.id,
    complete: r.complete,
    text: r.text,
    missing: r.missing,
    instruction: r.instruction,
    values: r.values.map((v) => ({ name: v.name, value: v.value, source: v.source })),
  };
}

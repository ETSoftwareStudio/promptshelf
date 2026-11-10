import { type Tool } from "@raycast/api";

import { saveDraft } from "../lib/shelf";

type Input = {
  /** A short title. The prompt ID comes from it. */
  title: string;
  /** The prompt text, in Markdown. Use {{name}} for a blank. */
  text: string;
};

/** Saves a prompt as a draft in the user's Inbox. It can not write anywhere else. */
export default async function tool(input: Input) {
  const saved = await saveDraft(input.title.trim(), input.text);
  return { id: saved.id, kind: saved.kind, path: saved.path };
}

export const confirmation: Tool.Confirmation<Input> = async (input) => ({
  message: "Save this prompt as a draft in your PromptShelf Inbox?",
  info: [
    { name: "Title", value: input.title },
    { name: "Text", value: input.text.length > 400 ? `${input.text.slice(0, 400)}…` : input.text },
  ],
});

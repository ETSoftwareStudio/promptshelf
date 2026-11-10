import { Color, List } from "@raycast/api";
import { useCachedPromise } from "@raycast/utils";
import { homedir } from "node:os";

import { blankStates, tagText, type BlankState } from "../lib/blanks";
import { conversationColor, repoColor } from "../lib/colors";
import type { PromptSummary } from "../lib/contract";
import { renderFilled } from "../lib/markdown";
import { fillPrompt, showPrompt } from "../lib/shelf";

function tagColor(state: BlankState): Color.ColorLike {
  switch (state.tone) {
    case "conversation":
      return conversationColor;
    case "repo":
      return repoColor;
    default:
      return Color.SecondaryText;
  }
}

function tildify(path: string): string {
  const home = homedir();
  return path.startsWith(home) ? `~${path.slice(home.length)}` : path;
}

/**
 * The preview: `shelf show` for the template and its blanks, and for a prompt
 * with blanks, `shelf fill --no-run` for the values (no shell commands run
 * while you browse). A prompt without blanks is complete, and a complete fill
 * counts as a use, so it is shown from `show` alone.
 */
async function loadPreview(id: string, hasBlanks: boolean) {
  const [show, fill] = await Promise.all([
    showPrompt(id),
    hasBlanks ? fillPrompt(id, { run: false }) : Promise.resolve(undefined),
  ]);
  const states = blankStates(show.blanks, fill);
  const markdown = fill ? renderFilled(show.body, fill.text, states) : show.body;
  return { show, states, markdown };
}

export function PromptDetail({ prompt }: { prompt: PromptSummary }) {
  const { data, isLoading, error } = useCachedPromise(loadPreview, [prompt.id, prompt.blanks.length > 0], {
    keepPreviousData: false,
    // The error shows in the detail; no toast for each prompt you pass.
    onError: () => undefined,
  });
  const markdown = error ? `**Could not fill this prompt.**\n\n${error.message}` : data?.markdown;
  return (
    <List.Item.Detail
      isLoading={isLoading}
      markdown={markdown}
      metadata={
        <List.Item.Detail.Metadata>
          {data && data.states.length > 0 && (
            <List.Item.Detail.Metadata.TagList title="Blanks">
              {data.states.map((s) => (
                <List.Item.Detail.Metadata.TagList.Item key={s.name} text={tagText(s)} color={tagColor(s)} />
              ))}
            </List.Item.Detail.Metadata.TagList>
          )}
          <List.Item.Detail.Metadata.Label title="ID" text={prompt.id} />
          <List.Item.Detail.Metadata.Label title="Kind" text={prompt.kind} />
          {prompt.tags.length > 0 && (
            <List.Item.Detail.Metadata.TagList title="Tags">
              {prompt.tags.map((t) => (
                <List.Item.Detail.Metadata.TagList.Item key={t} text={t} />
              ))}
            </List.Item.Detail.Metadata.TagList>
          )}
          {prompt.pinned !== undefined && (
            <List.Item.Detail.Metadata.Label title="Pinned" text={`Position ${prompt.pinned}`} />
          )}
          <List.Item.Detail.Metadata.Label title="File" text={tildify(prompt.path)} />
        </List.Item.Detail.Metadata>
      }
    />
  );
}

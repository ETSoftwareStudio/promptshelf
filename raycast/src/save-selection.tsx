import { Action, ActionPanel, Form, getSelectedText, Icon, showToast, Toast, popToRoot } from "@raycast/api";
import { useEffect, useState } from "react";

import { InstallView } from "./components/SetupViews";
import { ShelfMissingError } from "./lib/binary";
import { showFailure } from "./lib/feedback";
import { saveDraft, shelfBinary } from "./lib/shelf";
import { titleFrom } from "./lib/title";

/** Save Selection as Prompt: the selected text becomes a draft in the Inbox. */
export default function SaveSelection() {
  const [text, setText] = useState("");
  const [title, setTitle] = useState("");
  const [loading, setLoading] = useState(true);
  const [missing, setMissing] = useState(false);

  useEffect(() => {
    try {
      shelfBinary();
    } catch (error) {
      if (error instanceof ShelfMissingError) setMissing(true);
    }
    getSelectedText()
      .then((selected) => {
        setText(selected);
        setTitle(titleFrom(selected));
      })
      .catch(() =>
        showToast({ style: Toast.Style.Failure, title: "No text selected", message: "Type or paste the prompt" }),
      )
      .finally(() => setLoading(false));
  }, []);

  if (missing) return <InstallView />;

  async function save(values: { title: string; text: string }) {
    if (!values.title.trim() || !values.text.trim()) {
      await showToast({ style: Toast.Style.Failure, title: "A title and the text are needed" });
      return;
    }
    try {
      const saved = await saveDraft(values.title.trim(), values.text);
      await showToast({ style: Toast.Style.Success, title: "Saved to the Inbox", message: saved.id });
      await popToRoot();
    } catch (error) {
      await showFailure(error, "Could not save");
    }
  }

  return (
    <Form
      isLoading={loading}
      actions={
        <ActionPanel>
          <Action.SubmitForm title="Save to Inbox" icon={Icon.Tray} onSubmit={save} />
        </ActionPanel>
      }
    >
      <Form.TextField id="title" title="Title" value={title} onChange={setTitle} placeholder="Strict PR review" />
      <Form.TextArea id="text" title="Prompt" value={text} onChange={setText} enableMarkdown={false} />
      <Form.Description text="Saved as a draft in your Personal library's Inbox. Move it into the library from the PromptShelf app." />
    </Form>
  );
}

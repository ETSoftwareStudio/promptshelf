import { Action, ActionPanel, Form, Icon, showToast, Toast } from "@raycast/api";
import { useEffect, useState } from "react";
import { homedir } from "node:os";

import { blankStates, sourceNote } from "../lib/blanks";
import type { Blank, FillResult } from "../lib/contract";
import { deliver, frontAppName, showFailure, type Delivery } from "../lib/feedback";
import { fillPrompt, workingFolder } from "../lib/shelf";

export interface FillFormProps {
  title: string;
  /** The blanks from `shelf show`, for their order and specs. */
  blanks: Blank[];
  /** The fill that left blanks open (exit code 3). */
  fill: FillResult;
}

/** The Fill form. It opens only when a blank needs you. */
export function FillForm({ title, blanks, fill }: FillFormProps) {
  const states = blankStates(blanks, fill);
  const [values, setValues] = useState<Record<string, string>>(() =>
    Object.fromEntries(states.map((s) => [s.name, s.value?.value ?? s.spec.default ?? s.spec.choices?.[0] ?? ""])),
  );
  // A field keeps its type while you type: a value with several lines gets a text area.
  const [multiline] = useState(() => new Set(states.filter((s) => s.value?.value.includes("\n")).map((s) => s.name)));
  const [errors, setErrors] = useState<Record<string, string | undefined>>({});
  const [app, setApp] = useState<string>();
  useEffect(() => {
    void frontAppName().then(setApp);
  }, []);

  async function submit(how: Delivery) {
    const empty = states.filter((s) => !s.value && !values[s.name]?.trim()).map((s) => s.name);
    if (empty.length > 0) {
      setErrors(Object.fromEntries(empty.map((n) => [n, "Type a value"])));
      return;
    }
    const toast = await showToast({ style: Toast.Style.Animated, title: "Filling" });
    try {
      const result = await fillPrompt(fill.id, { values });
      if (!result.complete) {
        toast.style = Toast.Style.Failure;
        toast.title = "Some blanks are still open";
        toast.message = result.missing.map((m) => m.name).join(", ");
        return;
      }
      await toast.hide();
      await deliver(result.text, how);
    } catch (error) {
      await toast.hide();
      await showFailure(error);
    }
  }

  const folder = workingFolder();
  const home = homedir();

  return (
    <Form
      navigationTitle={`Fill ${title}`}
      actions={
        <ActionPanel>
          <Action.SubmitForm
            title={app ? `Paste into ${app}` : "Paste into Front App"}
            icon={Icon.Clipboard}
            onSubmit={() => submit("paste")}
          />
          <Action.SubmitForm title="Copy Filled Prompt" icon={Icon.CopyClipboard} onSubmit={() => submit("copy")} />
        </ActionPanel>
      }
    >
      {states.flatMap((s) => {
        const common = {
          id: s.name,
          title: s.name,
          value: values[s.name] ?? "",
          error: errors[s.name],
          onChange: (v: string) => {
            setValues((old) => ({ ...old, [s.name]: v }));
            if (errors[s.name]) setErrors((old) => ({ ...old, [s.name]: undefined }));
          },
        };
        const field = s.spec.choices?.length ? (
          <Form.Dropdown key={s.name} {...common}>
            {s.spec.choices.map((c) => (
              <Form.Dropdown.Item key={c} title={c} value={c} />
            ))}
          </Form.Dropdown>
        ) : multiline.has(s.name) ? (
          <Form.TextArea key={s.name} {...common} enableMarkdown={false} />
        ) : (
          <Form.TextField key={s.name} {...common} placeholder={s.spec.hint} />
        );
        return [field, <Form.Description key={`${s.name}-note`} text={sourceNote(s, folder, home)} />];
      })}
    </Form>
  );
}

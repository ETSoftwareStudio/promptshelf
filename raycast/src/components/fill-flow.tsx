import { showToast, Toast } from "@raycast/api";
import type { ReactNode } from "react";

import { deliver, showFailure, type Delivery } from "../lib/feedback";
import { fillPrompt, showPrompt } from "../lib/shelf";
import { FillForm } from "./FillForm";

/**
 * Fills a prompt with every resolver on. When it is complete, pastes or copies
 * it at once. When blanks are open (exit code 3), opens the Fill form.
 */
export async function fillAndDeliver(
  prompt: { id: string; title: string },
  how: Delivery,
  push: (view: ReactNode) => void,
): Promise<void> {
  const toast = await showToast({ style: Toast.Style.Animated, title: "Filling", message: prompt.title });
  try {
    const result = await fillPrompt(prompt.id);
    if (result.complete) {
      await toast.hide();
      await deliver(result.text, how);
      return;
    }
    const show = await showPrompt(prompt.id);
    await toast.hide();
    push(<FillForm title={prompt.title} blanks={show.blanks} fill={result} />);
  } catch (error) {
    await toast.hide();
    await showFailure(error);
  }
}

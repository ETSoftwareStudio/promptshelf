// Toasts for failures, and the hand-off of a filled prompt.

import { Clipboard, closeMainWindow, getFrontmostApplication, showHUD, showToast, Toast } from "@raycast/api";

import { ShelfMissingError } from "./binary";
import { ContractError } from "./contract";
import { ShelfCommandError } from "./client";

/** Shows a failure toast for any error from a `shelf` call. */
export async function showFailure(error: unknown, title?: string): Promise<void> {
  if (error instanceof ShelfCommandError) {
    await showToast({
      style: Toast.Style.Failure,
      title: title ?? error.problem.title,
      message: error.problem.message,
    });
  } else if (error instanceof ShelfMissingError) {
    await showToast({ style: Toast.Style.Failure, title: "shelf is not installed", message: error.message });
  } else if (error instanceof ContractError) {
    await showToast({
      style: Toast.Style.Failure,
      title: "Update PromptShelf or this extension",
      message: error.message,
    });
  } else {
    await showToast({
      style: Toast.Style.Failure,
      title: title ?? "Something went wrong",
      message: error instanceof Error ? error.message : String(error),
    });
  }
}

/** True when shelf has no config yet, so the person must run `shelf init`. */
export function needsInit(error: unknown): boolean {
  return error instanceof ShelfCommandError && error.body?.code === "no_config";
}

export type Delivery = "paste" | "copy";

/** Pastes the text into the app that was in front of Raycast, or copies it. */
export async function deliver(text: string, how: Delivery): Promise<void> {
  if (how === "copy") {
    await Clipboard.copy(text);
    await showHUD("Copied the filled prompt");
    return;
  }
  await closeMainWindow();
  await Clipboard.paste(text);
}

/** The name of the app a paste goes to, for action titles. */
export async function frontAppName(): Promise<string | undefined> {
  try {
    return (await getFrontmostApplication()).name;
  } catch {
    return undefined;
  }
}

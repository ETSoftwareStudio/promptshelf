// The exit codes of `shelf`, and the message to show for each.

import type { ShelfErrorBody } from "./contract";

export const Exit = {
  Ok: 0,
  Error: 1,
  Usage: 2,
  BlanksOpen: 3,
  NotFound: 4,
  Refused: 5,
} as const;

export type ExitCode = (typeof Exit)[keyof typeof Exit];

export interface Problem {
  title: string;
  message?: string;
}

/** True for the exit codes where stdout holds a usable result. */
export function isResult(code: number): boolean {
  return code === Exit.Ok || code === Exit.BlanksOpen;
}

/** The toast title and message for a failed command. */
export function problemFor(code: number, error?: ShelfErrorBody, stderr?: string): Problem {
  const detail = error?.message || lastLine(stderr);
  switch (code) {
    case Exit.Usage:
      return { title: "shelf did not accept the input", message: detail };
    case Exit.BlanksOpen:
      return { title: "Some blanks are open", message: detail };
    case Exit.NotFound:
      if (error?.matches && error.matches.length > 1) {
        return { title: "More than one prompt matches", message: error.matches.join(", ") };
      }
      return { title: "Prompt not found", message: detail };
    case Exit.Refused:
      return { title: "shelf refused the change", message: detail };
    case Exit.Error:
      return { title: "shelf failed", message: detail };
    default:
      return { title: `shelf stopped with exit code ${code}`, message: detail };
  }
}

function lastLine(text?: string): string | undefined {
  const line = text
    ?.split("\n")
    .map((l) => l.trim())
    .filter(Boolean)
    .pop();
  return line?.replace(/^shelf: (error: )?/, "");
}

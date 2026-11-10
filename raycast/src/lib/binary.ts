// Finding the `shelf` binary. The extension never bundles or downloads it.

import { accessSync, constants, statSync } from "node:fs";
import { join } from "node:path";

export const INSTALL_COMMAND = "brew install etsoftwarestudio/tap/promptshelf";
export const CARGO_INSTALL_COMMAND = "cargo install promptshelf";

/** The places to look, in order: the preference, then the usual install folders. */
export function candidates(preference: string | undefined, home: string): string[] {
  const pref = preference?.trim();
  if (pref) return [pref.startsWith("~/") ? join(home, pref.slice(2)) : pref];
  return ["/opt/homebrew/bin/shelf", "/usr/local/bin/shelf", join(home, ".cargo/bin/shelf")];
}

export function isExecutableFile(path: string): boolean {
  try {
    if (!statSync(path).isFile()) return false;
    accessSync(path, constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

/** The first candidate that is an executable file, or undefined. */
export function findShelf(
  preference: string | undefined,
  home: string,
  isExecutable: (path: string) => boolean = isExecutableFile,
): string | undefined {
  return candidates(preference, home).find(isExecutable);
}

/** `shelf` was not found, or could not start. */
export class ShelfMissingError extends Error {
  constructor(public readonly looked: string[]) {
    super(`shelf was not found. Looked in: ${looked.join(", ")}`);
    this.name = "ShelfMissingError";
  }
}

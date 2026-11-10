// Running `shelf` with execFile: an argument list, never a shell string.

import { execFile } from "node:child_process";
import { dirname } from "node:path";

import { ShelfMissingError } from "./binary";

export interface RunOptions {
  /** Text for stdin. */
  input?: string;
  cwd?: string;
  timeoutMs?: number;
}

export interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

const MAX_BUFFER = 16 * 1024 * 1024;

/** The child's PATH: git and the tools a `from:` command needs, which Raycast's own PATH lacks. */
export function childPath(bin: string, current: string | undefined): string {
  const dirs = [dirname(bin), "/opt/homebrew/bin", "/usr/local/bin", ...(current ?? "/usr/bin:/bin").split(":")];
  return [...new Set(dirs.filter(Boolean))].join(":");
}

export function runShelf(bin: string, args: string[], options: RunOptions = {}): Promise<RunResult> {
  return new Promise((resolve, reject) => {
    const child = execFile(
      bin,
      args,
      {
        cwd: options.cwd,
        timeout: options.timeoutMs ?? 30_000,
        maxBuffer: MAX_BUFFER,
        encoding: "utf8",
        env: { ...process.env, PATH: childPath(bin, process.env.PATH) },
      },
      (error, stdout, stderr) => {
        if (!error) return resolve({ code: 0, stdout, stderr });
        const code = (error as NodeJS.ErrnoException).code;
        if (code === "ENOENT" || code === "EACCES") return reject(new ShelfMissingError([bin]));
        if (typeof code === "number") return resolve({ code, stdout, stderr });
        if (error.killed) return reject(new Error(`shelf did not finish in ${(options.timeoutMs ?? 30_000) / 1000} s`));
        reject(error);
      },
    );
    child.stdin?.on("error", () => undefined);
    child.stdin?.end(options.input ?? "");
  });
}

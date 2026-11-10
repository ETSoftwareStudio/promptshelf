// Reads the CLI's own JSON snapshots, so these tests break when the contract changes.

import { readFileSync } from "node:fs";
import { join } from "node:path";

const DIR = join(__dirname, "..", "..", "crates", "promptshelf", "tests", "snapshots");

/** The JSON body of `json_contract__<name>.snap`, after the insta header. */
export function snapshot(name: string): string {
  const text = readFileSync(join(DIR, `json_contract__${name}.snap`), "utf8");
  const parts = text.split(/^---$/m);
  return parts.slice(2).join("---").trim();
}

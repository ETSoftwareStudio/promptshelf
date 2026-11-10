import { expect, test } from "vite-plus/test";
import { mergeValues } from "./filler.svelte";

test("typed values win and keep the given source", () => {
  const out = mergeValues(
    [
      { name: "diff", value: "+1", source: "command", detail: "git diff" },
      { name: "audience", value: "developers", source: "default" },
    ],
    { audience: "managers", focus: "retries" },
  );
  expect(out).toEqual({
    diff: { value: "+1", source: "command", detail: "git diff" },
    audience: { value: "managers", source: "given" },
    focus: { value: "retries", source: "given" },
  });
});

test("an empty typed value leaves the blank open", () => {
  const out = mergeValues([{ name: "audience", value: "developers", source: "default" }], {
    audience: "",
  });
  expect(out).toEqual({});
});

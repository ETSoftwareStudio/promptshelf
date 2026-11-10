import { expect, test } from "vite-plus/test";
import { libraryKey } from "./keys";

const k = (
  key: string,
  mods: { meta?: boolean; ctrl?: boolean; alt?: boolean; shift?: boolean } = {},
) => ({
  key,
  metaKey: mods.meta ?? false,
  ctrlKey: mods.ctrl ?? false,
  altKey: mods.alt ?? false,
  shiftKey: mods.shift ?? false,
});

test("⌘1 to ⌘9 open pins, with ⌘ or Ctrl", () => {
  expect(libraryKey(k("3", { meta: true }))).toEqual({ pin: 3 });
  expect(libraryKey(k("9", { ctrl: true }))).toEqual({ pin: 9 });
  expect(libraryKey(k("0", { meta: true }))).toBeNull();
});

test("other shortcuts", () => {
  expect(libraryKey(k("f", { meta: true }))).toBe("find");
  expect(libraryKey(k("Enter", { meta: true }))).toBe("send");
  expect(libraryKey(k("n", { ctrl: true }))).toBe("new");
  expect(libraryKey(k("e", { meta: true }))).toBe("edit");
});

test("needs the modifier alone", () => {
  expect(libraryKey(k("f"))).toBeNull();
  expect(libraryKey(k("f", { meta: true, shift: true }))).toBeNull();
  expect(libraryKey(k("1", { meta: true, alt: true }))).toBeNull();
});

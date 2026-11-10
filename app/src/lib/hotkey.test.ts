import { describe, expect, test } from "vite-plus/test";
import { acceleratorFrom, displayAccelerator, modKey, type KeyLike } from "./hotkey";

const press = (code: string, mods: Partial<KeyLike> = {}): KeyLike => ({
  key: code.replace(/^Key/, "").toLowerCase(),
  code,
  metaKey: false,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  ...mods,
});

describe("acceleratorFrom", () => {
  test("uses ⌘ on a Mac and Ctrl elsewhere as CommandOrControl", () => {
    expect(acceleratorFrom(press("Space", { metaKey: true, shiftKey: true }), true)).toBe(
      "CommandOrControl+Shift+Space",
    );
    expect(acceleratorFrom(press("KeyP", { ctrlKey: true, altKey: true }), false)).toBe(
      "CommandOrControl+Alt+P",
    );
  });

  test("needs a modifier, except for function keys", () => {
    expect(acceleratorFrom(press("KeyP"), true)).toBeNull();
    expect(acceleratorFrom(press("KeyP", { shiftKey: true }), true)).toBeNull();
    expect(acceleratorFrom(press("F5"), false)).toBe("F5");
  });

  test("ignores a modifier pressed alone", () => {
    expect(
      acceleratorFrom({ ...press("ShiftLeft"), key: "Shift", shiftKey: true }, true),
    ).toBeNull();
  });

  test("keeps Control apart from ⌘ on a Mac", () => {
    expect(acceleratorFrom(press("Digit1", { ctrlKey: true }), true)).toBe("Control+1");
  });
});

describe("displayAccelerator", () => {
  test("shows symbols on a Mac", () => {
    expect(displayAccelerator("CommandOrControl+Shift+Space", true)).toBe("⌘⇧Space");
  });
  test("shows words elsewhere", () => {
    expect(displayAccelerator("CommandOrControl+Shift+Space", false)).toBe("Ctrl+Shift+Space");
  });
  test("an empty shortcut is off", () => {
    expect(displayAccelerator("", true)).toBe("Off");
  });
  test("modKey", () => {
    expect(modKey("1", true)).toBe("⌘1");
    expect(modKey("1", false)).toBe("Ctrl+1");
  });
});

// Shortcuts in Tauri's accelerator format ("CommandOrControl+Shift+Space"),
// recorded from key presses and shown the way each system writes them.

export interface KeyLike {
  key: string;
  code: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

const MODIFIER_KEYS = ["Meta", "Control", "Alt", "Shift", "OS"];

/** The key part of an accelerator, from `KeyboardEvent.code`, or null for a
 *  key that can not be part of a shortcut. */
function keyName(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code;
  const named: Record<string, string> = {
    Space: "Space",
    Enter: "Enter",
    Tab: "Tab",
    Backquote: "`",
    Minus: "-",
    Equal: "=",
    BracketLeft: "[",
    BracketRight: "]",
    Backslash: "\\",
    Semicolon: ";",
    Quote: "'",
    Comma: ",",
    Period: ".",
    Slash: "/",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
  };
  return named[code] ?? null;
}

/** Turns a key press into an accelerator. Returns null until the press has a
 *  modifier and a real key, so "Shift" alone or a plain letter is not a
 *  shortcut. On a Mac ⌘ is "CommandOrControl"; elsewhere Ctrl is. */
export function acceleratorFrom(e: KeyLike, mac: boolean): string | null {
  if (MODIFIER_KEYS.includes(e.key)) return null;
  const key = keyName(e.code);
  if (!key) return null;
  const parts: string[] = [];
  const primary = mac ? e.metaKey : e.ctrlKey;
  if (primary) parts.push("CommandOrControl");
  if (mac && e.ctrlKey) parts.push("Control");
  if (!mac && e.metaKey) parts.push("Super");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  const hasModifier = parts.some((p) => p !== "Shift");
  if (!hasModifier && !/^F\d+$/.test(key)) return null;
  parts.push(key);
  return parts.join("+");
}

/** An accelerator the way the system shows it: "⌘⇧Space" on a Mac,
 *  "Ctrl+Shift+Space" elsewhere. */
export function displayAccelerator(accel: string, mac: boolean): string {
  if (!accel.trim()) return "Off";
  const parts = accel.split("+");
  const mapMac: Record<string, string> = {
    CommandOrControl: "⌘",
    CmdOrCtrl: "⌘",
    Command: "⌘",
    Cmd: "⌘",
    Super: "⌘",
    Control: "⌃",
    Ctrl: "⌃",
    Alt: "⌥",
    Option: "⌥",
    Shift: "⇧",
  };
  const mapOther: Record<string, string> = {
    CommandOrControl: "Ctrl",
    CmdOrCtrl: "Ctrl",
    Control: "Ctrl",
    Command: "Super",
    Cmd: "Super",
  };
  if (mac) {
    return parts.map((p) => mapMac[p] ?? p).join("");
  }
  return parts.map((p) => mapOther[p] ?? p).join("+");
}

export const isMac = typeof navigator !== "undefined" && /Mac/i.test(navigator.platform);

/** "⌘1" on a Mac, "Ctrl+1" elsewhere. */
export function modKey(key: string, mac = isMac): string {
  return mac ? `⌘${key}` : `Ctrl+${key}`;
}

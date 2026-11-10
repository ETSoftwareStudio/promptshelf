// The library window's shortcuts. ⌘ on a Mac, Ctrl elsewhere.

export type LibraryKey = { pin: number } | "find" | "new" | "send" | "edit" | null;

export function libraryKey(e: {
  key: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}): LibraryKey {
  if (!(e.metaKey || e.ctrlKey) || e.altKey) return null;
  if (/^[1-9]$/.test(e.key) && !e.shiftKey) return { pin: Number(e.key) };
  if (e.shiftKey) return null;
  switch (e.key) {
    case "f":
      return "find";
    case "n":
      return "new";
    case "Enter":
      return "send";
    case "e":
      return "edit";
    default:
      return null;
  }
}

// The widths of the library window's sidebar and list. You drag the
// dividers to change them, and the window keeps them for the next start.

export type Columns = { side: number; list: number; sideHidden: boolean };

export const SIDE = { min: 160, max: 320, initial: 200 };
export const LIST = { min: 220, max: 520, initial: 280 };
/** The detail keeps this width at least, so a wide list can not hide it. */
export const DETAIL_MIN = 360;
/** Drag the sidebar narrower than this and it hides, like a Finder sidebar. */
export const SIDE_COLLAPSE = 110;

export const INITIAL: Columns = { side: SIDE.initial, list: LIST.initial, sideHidden: false };

const KEY = "promptshelf.columns";

const clamp = (n: number, lo: number, hi: number) => Math.min(Math.max(n, lo), Math.max(lo, hi));

/** The list width for a drag, within its limits and the room the window has. */
export function listWidth(want: number, side: number, window: number): number {
  return Math.round(clamp(want, LIST.min, Math.min(LIST.max, window - side - DETAIL_MIN)));
}

/** The sidebar for a drag: hidden below SIDE_COLLAPSE, else within its limits. */
export function sideWidth(want: number): { side: number; hidden: boolean } {
  if (want < SIDE_COLLAPSE) return { side: SIDE.min, hidden: true };
  return { side: Math.round(clamp(want, SIDE.min, SIDE.max)), hidden: false };
}

/** Reads the saved widths. Bad or missing values give the initial widths. */
export function parseColumns(raw: string | null): Columns {
  try {
    const v = JSON.parse(raw ?? "null") as Partial<Columns> | null;
    if (!v || typeof v !== "object") return { ...INITIAL };
    const num = (n: unknown, d: number) => (typeof n === "number" && Number.isFinite(n) ? n : d);
    return {
      side: Math.round(clamp(num(v.side, SIDE.initial), SIDE.min, SIDE.max)),
      list: Math.round(clamp(num(v.list, LIST.initial), LIST.min, LIST.max)),
      sideHidden: v.sideHidden === true,
    };
  } catch {
    return { ...INITIAL };
  }
}

export function loadColumns(): Columns {
  try {
    return parseColumns(localStorage.getItem(KEY));
  } catch {
    return { ...INITIAL };
  }
}

export function saveColumns(c: Columns): void {
  try {
    localStorage.setItem(KEY, JSON.stringify(c));
  } catch {
    // The widths are a convenience. Without storage they reset at start.
  }
}

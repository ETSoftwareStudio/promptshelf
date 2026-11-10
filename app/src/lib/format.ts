// Small text helpers for the list and the detail.

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/** A `YYYY-MM-DD` day (UTC) as "Today", "Yesterday" or "Oct 28". */
export function shortDay(day: string | null | undefined, today = new Date()): string {
  if (!day) return "";
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(day);
  if (!m) return day;
  const [y, mo, d] = [Number(m[1]), Number(m[2]), Number(m[3])];
  const t = Date.UTC(today.getUTCFullYear(), today.getUTCMonth(), today.getUTCDate());
  const days = Math.round((t - Date.UTC(y, mo - 1, d)) / 86_400_000);
  if (days === 0) return "Today";
  if (days === 1) return "Yesterday";
  const label = `${MONTHS[mo - 1]} ${d}`;
  return y === today.getUTCFullYear() ? label : `${label}, ${y}`;
}

export function usedTimes(n: number): string {
  if (n === 0) return "not used yet";
  if (n === 1) return "used once";
  return `used ${n} times`;
}

/** The kinds in sidebar order, with their names. */
export const KINDS = [
  { kind: "template", label: "Templates", icon: "braces" },
  { kind: "snippet", label: "Snippets", icon: "text" },
  { kind: "context", label: "Context", icon: "person" },
  { kind: "persona", label: "Personas", icon: "person" },
  { kind: "workflow", label: "Workflows", icon: "braces" },
] as const;

/** A library name for people: "acme-eng" → "Acme eng". */
export function libraryLabel(name: string): string {
  const words = name.replace(/-/g, " ");
  return words.charAt(0).toUpperCase() + words.slice(1);
}

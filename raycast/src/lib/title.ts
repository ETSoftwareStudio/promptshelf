/** A title from the first line of the text: at most 8 words, without Markdown marks. */
export function titleFrom(text: string): string {
  const line = text.trim().split("\n")[0] ?? "";
  const words = line
    .replace(/[#>*_`]/g, "")
    .trim()
    .split(/\s+/)
    .filter(Boolean);
  return words.slice(0, 8).join(" ");
}

// The source colours from the PromptShelf palette. OKLCH is the source of truth;
// Raycast tags take hex, so this converts once at load.

export interface Oklch {
  l: number;
  c: number;
  h: number;
}

/** OKLCH to an sRGB hex string, clipped to the sRGB gamut. */
export function oklchToHex({ l, c, h }: Oklch): string {
  const rad = (h * Math.PI) / 180;
  const a = c * Math.cos(rad);
  const b = c * Math.sin(rad);
  const l_ = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m_ = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s_ = (l - 0.0894841775 * a - 1.291485548 * b) ** 3;
  const linear = [
    4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
    -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
    -0.0041960863 * l_ - 0.7034186147 * m_ + 1.707614701 * s_,
  ];
  return (
    "#" +
    linear
      .map((x) => {
        const v = x <= 0.0031308 ? 12.92 * x : 1.055 * Math.max(x, 0) ** (1 / 2.4) - 0.055;
        const byte = Math.round(Math.min(1, Math.max(0, v)) * 255);
        return byte.toString(16).padStart(2, "0");
      })
      .join("")
  );
}

export interface ThemedColor {
  light: string;
  dark: string;
}

function themed(light: Oklch, dark: Oklch): ThemedColor {
  return { light: oklchToHex(light), dark: oklchToHex(dark) };
}

/** Rose: a value from the conversation. */
export const conversationColor = themed({ l: 0.52, c: 0.17, h: 355 }, { l: 0.76, c: 0.13, h: 355 });
/** Green: a value from the repo or its files (branch, git commands). */
export const repoColor = themed({ l: 0.48, c: 0.11, h: 158 }, { l: 0.76, c: 0.12, h: 158 });
/** Pin marks. */
export const pinColor = themed({ l: 0.56, c: 0.17, h: 35 }, { l: 0.74, c: 0.14, h: 45 });

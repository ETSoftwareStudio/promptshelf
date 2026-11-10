import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { conversationColor, oklchToHex, repoColor } from "../src/lib/colors";

const rgb = (hex: string) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));

describe("oklchToHex", () => {
  it("converts white, black and a known colour", () => {
    assert.equal(oklchToHex({ l: 1, c: 0, h: 0 }), "#ffffff");
    assert.equal(oklchToHex({ l: 0, c: 0, h: 0 }), "#000000");
    // Tailwind's red-500 is oklch(63.7% 0.237 25.331) = #fb2c36.
    assert.equal(oklchToHex({ l: 0.637, c: 0.237, h: 25.331 }), "#fb2c36");
  });

  it("gives the palette's source colours", () => {
    assert.match(conversationColor.light, /^#[0-9a-f]{6}$/);
    assert.match(repoColor.dark, /^#[0-9a-f]{6}$/);
    // Rose leans red, green leans green.
    const [r, g] = rgb(conversationColor.light);
    assert.ok(r! > g!);
    const [r2, g2] = rgb(repoColor.light);
    assert.ok(g2! > r2!);
  });
});

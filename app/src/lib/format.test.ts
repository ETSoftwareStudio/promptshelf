import { expect, test } from "vite-plus/test";
import { libraryLabel, shortDay, usedTimes } from "./format";

const today = new Date(Date.UTC(2026, 10, 9, 15));

test("shortDay", () => {
  expect(shortDay("2026-11-09", today)).toBe("Today");
  expect(shortDay("2026-11-08", today)).toBe("Yesterday");
  expect(shortDay("2026-10-28", today)).toBe("Oct 28");
  expect(shortDay("2025-12-31", today)).toBe("Dec 31, 2025");
  expect(shortDay(null, today)).toBe("");
});

test("usedTimes", () => {
  expect(usedTimes(0)).toBe("not used yet");
  expect(usedTimes(1)).toBe("used once");
  expect(usedTimes(23)).toBe("used 23 times");
});

test("libraryLabel", () => {
  expect(libraryLabel("acme-eng")).toBe("Acme eng");
  expect(libraryLabel("personal")).toBe("Personal");
});

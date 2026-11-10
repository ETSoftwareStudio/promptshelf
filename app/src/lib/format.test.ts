import { expect, test } from "vite-plus/test";
import { libraryLabel, proposalLabel, shortDay, usedTimes } from "./format";

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

test("proposalLabel", () => {
  expect(proposalLabel({ status: "not-sent", unsent: true })).toEqual({
    chip: "Not sent",
    text: "Only on this computer",
  });
  expect(proposalLabel({ status: "sent", number: 42, unsent: false })).toEqual({
    chip: "#42",
    tone: "blue",
    text: "Waiting for review",
  });
  expect(proposalLabel({ status: "sent", unsent: true }).text).toBe(
    "Waiting for review · new edits not sent",
  );
  expect(proposalLabel({ status: "changes-requested", number: 7, unsent: false }).tone).toBe(
    "warn",
  );
});

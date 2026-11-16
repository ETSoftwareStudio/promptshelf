import { describe, expect, it } from "vite-plus/test";
import { DETAIL_MIN, INITIAL, LIST, SIDE, listWidth, parseColumns, sideWidth } from "./columns";

describe("listWidth", () => {
  it("keeps the list within its limits", () => {
    expect(listWidth(100, 200, 1400)).toBe(LIST.min);
    expect(listWidth(900, 200, 1400)).toBe(LIST.max);
    expect(listWidth(300.4, 200, 1400)).toBe(300);
  });

  it("leaves the detail its minimum width", () => {
    expect(listWidth(500, 200, 900)).toBe(900 - 200 - DETAIL_MIN);
  });

  it("never goes below the minimum in a narrow window", () => {
    expect(listWidth(500, 300, 760)).toBe(LIST.min);
  });
});

describe("sideWidth", () => {
  it("hides the sidebar on a drag past the collapse point", () => {
    expect(sideWidth(60)).toEqual({ side: SIDE.min, hidden: true });
  });

  it("keeps a shown sidebar within its limits", () => {
    expect(sideWidth(130)).toEqual({ side: SIDE.min, hidden: false });
    expect(sideWidth(999)).toEqual({ side: SIDE.max, hidden: false });
    expect(sideWidth(240)).toEqual({ side: 240, hidden: false });
  });
});

describe("parseColumns", () => {
  it("gives the initial widths for missing or bad data", () => {
    expect(parseColumns(null)).toEqual(INITIAL);
    expect(parseColumns("not json")).toEqual(INITIAL);
    expect(parseColumns('"text"')).toEqual(INITIAL);
    expect(parseColumns('{"side":"wide"}')).toEqual(INITIAL);
  });

  it("clamps saved widths that are out of range", () => {
    expect(parseColumns('{"side":10,"list":4000,"sideHidden":true}')).toEqual({
      side: SIDE.min,
      list: LIST.max,
      sideHidden: true,
    });
  });
});

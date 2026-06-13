import { describe, it, expect } from "vitest";
import { level, buildYearGrid } from "./heatmap";

describe("level", () => {
  it("is 0 for zero value or zero max", () => {
    expect(level(0, 100)).toBe(0);
    expect(level(50, 0)).toBe(0);
  });
  it("buckets into quartiles", () => {
    expect(level(10, 100)).toBe(1);
    expect(level(40, 100)).toBe(2);
    expect(level(60, 100)).toBe(3);
    expect(level(100, 100)).toBe(4);
  });
});

describe("buildYearGrid", () => {
  it("returns 53 weeks of 7 days each, Monday-aligned, ending >= today", () => {
    const today = new Date(2026, 5, 13); // 13 Jun 2026 (local)
    const grid = buildYearGrid(today);
    expect(grid.length).toBe(53);
    expect(grid.every((w) => w.length === 7)).toBe(true);
    // First cell is a Monday (getDay() === 1)
    expect(grid[0][0].getDay()).toBe(1);
    // Last column contains today
    const flat = grid.flat().map((d) => d.toDateString());
    expect(flat).toContain(today.toDateString());
  });
});

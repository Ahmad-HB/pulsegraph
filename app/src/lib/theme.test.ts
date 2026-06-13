import { describe, it, expect } from "vitest";
import { PRESETS, themeToVars, defaultTheme, type Theme } from "./theme";

describe("theme", () => {
  it("has the four documented presets", () => {
    expect(Object.keys(PRESETS)).toEqual(
      expect.arrayContaining(["github-green", "blue", "purple", "amber"]),
    );
    expect(PRESETS["github-green"]).toHaveLength(5);
  });

  it("defaultTheme is github-green dark", () => {
    expect(defaultTheme.palette).toBe("github-green");
    expect(defaultTheme.mode).toBe("dark");
  });

  it("themeToVars maps a preset to 5 ramp vars + surface vars", () => {
    const vars = themeToVars(defaultTheme);
    expect(vars["--hm-0"]).toBeDefined();
    expect(vars["--hm-4"]).toBeDefined();
    expect(vars["--bg"]).toBeDefined();
    expect(vars["--fg"]).toBeDefined();
  });

  it("custom palette uses the customRamp", () => {
    const t: Theme = { mode: "dark", palette: "custom", customRamp: ["#000000","#111111","#222222","#333333","#444444"] };
    const vars = themeToVars(t);
    expect(vars["--hm-4"]).toBe("#444444");
  });

  it("light mode flips bg/fg", () => {
    const dark = themeToVars({ ...defaultTheme, mode: "dark" });
    const light = themeToVars({ ...defaultTheme, mode: "light" });
    expect(light["--bg"]).not.toBe(dark["--bg"]);
  });
});

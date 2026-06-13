export type Mode = "dark" | "light";
export type PaletteName = "github-green" | "blue" | "purple" | "amber" | "custom";
export type Theme = { mode: Mode; palette: PaletteName; customRamp: string[] };

export const PRESETS: Record<Exclude<PaletteName, "custom">, string[]> = {
  "github-green": ["#161b22", "#0e4429", "#006d32", "#26a641", "#39d353"],
  blue:           ["#161b22", "#0a3069", "#0969da", "#218bff", "#54aeff"],
  purple:         ["#161b22", "#3c1e70", "#6639ba", "#8957e5", "#bc8cff"],
  amber:          ["#161b22", "#5a3e00", "#9e6a00", "#d4a72c", "#f2cc60"],
};

export const defaultTheme: Theme = {
  mode: "dark",
  palette: "github-green",
  customRamp: [...PRESETS["github-green"]],
};

const SURFACE = {
  dark:  { bg: "#0d1117", panel: "#161b22", border: "#21262d", fg: "#e6edf3", muted: "#7d8590", chip: "#161b22", chipBorder: "#30363d" },
  light: { bg: "#ffffff", panel: "#f6f8fa", border: "#d0d7de", fg: "#1f2328", muted: "#656d76", chip: "#f6f8fa", chipBorder: "#d0d7de" },
};

export function rampFor(theme: Theme): string[] {
  return theme.palette === "custom" ? theme.customRamp : PRESETS[theme.palette];
}

export function themeToVars(theme: Theme): Record<string, string> {
  const ramp = rampFor(theme);
  const s = SURFACE[theme.mode];
  return {
    "--hm-0": ramp[0], "--hm-1": ramp[1], "--hm-2": ramp[2], "--hm-3": ramp[3], "--hm-4": ramp[4],
    "--bg": s.bg, "--panel": s.panel, "--border": s.border, "--fg": s.fg, "--muted": s.muted,
    "--chip": s.chip, "--chip-border": s.chipBorder,
  };
}

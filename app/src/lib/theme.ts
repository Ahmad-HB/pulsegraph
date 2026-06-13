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
  dark:  { bg:"rgba(22,27,34,0.92)", panel:"rgba(255,255,255,0.05)", border:"rgba(255,255,255,0.09)",
           fg:"#e6edf3", muted:"#9aa4af", chip:"rgba(255,255,255,0.06)", chipBorder:"rgba(255,255,255,0.16)",
           empty:"#2d333b" },
  light: { bg:"rgba(248,250,252,0.94)", panel:"rgba(0,0,0,0.04)", border:"rgba(0,0,0,0.10)",
           fg:"#1f2328", muted:"#57606a", chip:"rgba(0,0,0,0.05)", chipBorder:"rgba(0,0,0,0.14)",
           empty:"#ebedf0" },
};

export function rampFor(theme: Theme): string[] {
  return theme.palette === "custom" ? theme.customRamp : PRESETS[theme.palette];
}

export function themeToVars(theme: Theme): Record<string, string> {
  const ramp = rampFor(theme);
  const s = SURFACE[theme.mode];
  return {
    "--hm-0": s.empty, "--hm-1": ramp[1], "--hm-2": ramp[2], "--hm-3": ramp[3], "--hm-4": ramp[4],
    "--bg": s.bg, "--panel": s.panel, "--border": s.border, "--fg": s.fg, "--muted": s.muted,
    "--chip": s.chip, "--chip-border": s.chipBorder,
  };
}

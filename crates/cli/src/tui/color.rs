//! Terminal color-capability detection and heatmap ramps.
//! Apple Terminal does not support 24-bit truecolor, so we degrade to xterm-256.

use ratatui::style::Color;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorMode {
    Truecolor,
    Indexed256,
    Mono,
}

impl ColorMode {
    /// Pure capability decision from inputs (testable without touching the environment).
    pub fn from_env(no_color: bool, colorterm: Option<&str>) -> ColorMode {
        if no_color {
            return ColorMode::Mono;
        }
        match colorterm {
            Some(v) if v.contains("truecolor") || v.contains("24bit") => ColorMode::Truecolor,
            _ => ColorMode::Indexed256,
        }
    }

    /// Detect from the real process environment.
    pub fn detect() -> ColorMode {
        let ct = std::env::var("COLORTERM").ok();
        ColorMode::from_env(std::env::var_os("NO_COLOR").is_some(), ct.as_deref())
    }
}

/// xterm-256 palette index for intensity level 0..=4 (0 = empty).
pub fn idx_ramp(lvl: u8) -> u8 {
    match lvl {
        0 => 235,
        1 => 22,
        2 => 28,
        3 => 34,
        _ => 40,
    }
}

/// Truecolor RGB for intensity level 0..=4.
pub fn rgb_ramp(lvl: u8) -> (u8, u8, u8) {
    match lvl {
        0 => (45, 51, 59),
        1 => (14, 68, 41),
        2 => (0, 109, 50),
        3 => (38, 166, 65),
        _ => (57, 211, 83),
    }
}

/// Monochrome glyph for intensity level 0..=4.
pub fn level_glyph(lvl: u8) -> &'static str {
    match lvl {
        0 => "·",
        1 => "░",
        2 => "▒",
        3 => "▓",
        _ => "█",
    }
}

/// ratatui background color for a heatmap cell (TUI path).
pub fn level_color(mode: ColorMode, lvl: u8) -> Color {
    match mode {
        ColorMode::Truecolor => {
            let (r, g, b) = rgb_ramp(lvl);
            Color::Rgb(r, g, b)
        }
        ColorMode::Indexed256 => Color::Indexed(idx_ramp(lvl)),
        ColorMode::Mono => Color::Reset,
    }
}

/// A 2-char colored cell as a raw ANSI string (static print path).
pub fn ansi_cell(mode: ColorMode, lvl: u8) -> String {
    match mode {
        ColorMode::Truecolor => {
            let (r, g, b) = rgb_ramp(lvl);
            format!("\x1b[48;2;{r};{g};{b}m  \x1b[0m")
        }
        ColorMode::Indexed256 => format!("\x1b[48;5;{}m  \x1b[0m", idx_ramp(lvl)),
        ColorMode::Mono => {
            let g = level_glyph(lvl);
            format!("{g}{g}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_color_forces_mono_even_with_truecolor() {
        assert_eq!(ColorMode::from_env(true, Some("truecolor")), ColorMode::Mono);
    }

    #[test]
    fn truecolor_detected_from_colorterm() {
        assert_eq!(ColorMode::from_env(false, Some("truecolor")), ColorMode::Truecolor);
        assert_eq!(ColorMode::from_env(false, Some("24bit")), ColorMode::Truecolor);
    }

    #[test]
    fn defaults_to_indexed_when_no_colorterm() {
        assert_eq!(ColorMode::from_env(false, None), ColorMode::Indexed256);
        assert_eq!(ColorMode::from_env(false, Some("")), ColorMode::Indexed256);
    }

    #[test]
    fn indexed_ramp_endpoints() {
        assert_eq!(idx_ramp(0), 235);
        assert_eq!(idx_ramp(4), 40);
    }

    #[test]
    fn ansi_cell_indexed_emits_256color_sequence() {
        assert_eq!(ansi_cell(ColorMode::Indexed256, 0), "\x1b[48;5;235m  \x1b[0m");
    }

    #[test]
    fn ansi_cell_mono_uses_glyphs_not_escapes() {
        assert_eq!(ansi_cell(ColorMode::Mono, 4), "██");
    }
}

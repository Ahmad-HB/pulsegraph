//! ratatui draw functions. Pure rendering from &App; no state mutation.

use chrono::{Datelike, Duration, NaiveDate};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::render::{fmt_value, level, metric_label};
use crate::tui::app::{App, Picker, PickerKind};
use crate::tui::color::{self, ColorMode};

const GUTTER: usize = 5;

pub fn draw(f: &mut Frame, app: &App, mode: ColorMode) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(9),    // heatmap
            Constraint::Length(8), // day detail
            Constraint::Length(3), // stats footer
        ])
        .split(f.area());

    draw_heatmap(f, app, mode, chunks[0]);
    draw_detail(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    if let Some(p) = &app.picker {
        draw_picker(f, p, f.area());
    }
}

fn title_line(app: &App) -> String {
    let proj = app.filter.project.clone().unwrap_or_else(|| "all projects".into());
    let model = app.filter.model.clone().unwrap_or_else(|| "all models".into());
    format!(
        " PulseGraph — {} · {} · {} · {} ",
        metric_label(app.metric),
        proj,
        model,
        app.range.label()
    )
}

fn month_abbr(m: u32) -> &'static str {
    ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
        [(m as usize).saturating_sub(1).min(11)]
}

fn weekday_label(row: usize) -> &'static str {
    match row {
        0 => "Mon  ",
        2 => "Wed  ",
        4 => "Fri  ",
        _ => "     ",
    }
}

fn cell_span(mode: ColorMode, lvl: u8, cursor: bool) -> Span<'static> {
    match mode {
        ColorMode::Mono => {
            let g = color::level_glyph(lvl);
            let style = if cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            Span::styled(format!("{g}{g}"), style)
        }
        _ => {
            let mut style = Style::default().bg(color::level_color(mode, lvl));
            if cursor {
                style = style.fg(Color::White).add_modifier(Modifier::REVERSED);
            }
            Span::styled("  ", style)
        }
    }
}

/// Month labels aligned to the cell grid (writes 3-letter names into a char buffer).
fn month_header(start: NaiveDate, weeks: i64) -> Line<'static> {
    let width = GUTTER + (weeks as usize) * 2;
    let mut buf = vec![b' '; width];
    let mut last = 0u32;
    for w in 0..weeks {
        let d = start + Duration::days(w * 7);
        if d.month() != last {
            last = d.month();
            let name = month_abbr(d.month());
            let col = GUTTER + (w as usize) * 2;
            for (i, ch) in name.bytes().enumerate() {
                if col + i < width {
                    buf[col + i] = ch;
                }
            }
        }
    }
    Line::from(String::from_utf8(buf).unwrap())
}

fn legend_line(mode: ColorMode) -> Line<'static> {
    let mut spans = vec![Span::raw(" ".repeat(GUTTER)), Span::raw("Less ")];
    for lvl in 0..=4u8 {
        spans.push(cell_span(mode, lvl, false));
    }
    spans.push(Span::raw(" More"));
    Line::from(spans)
}

fn draw_heatmap(f: &mut Frame, app: &App, mode: ColorMode, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(title_line(app));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let today = app.today;
    let full_start = app.grid_start();
    let total_weeks = (today - full_start).num_days() / 7 + 1;

    // Clamp the number of week-columns to what fits the inner width.
    let avail = inner.width as i64 - GUTTER as i64;
    let max_weeks = (avail / 2).max(1);
    let weeks = total_weeks.min(max_weeks).max(1);

    // Anchor the grid to the Monday of today's week so today is always in the last
    // column, then step back (weeks - 1) weeks. (Equals grid_start() when unclamped.)
    let current_monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let start = current_monday - Duration::days((weeks - 1) * 7);

    let max = app.max_value();

    let mut lines: Vec<Line> = Vec::new();
    lines.push(month_header(start, weeks));
    for row in 0..7usize {
        let mut spans = vec![Span::raw(weekday_label(row))];
        for w in 0..weeks {
            let date = start + Duration::days(w * 7 + row as i64);
            if date > today {
                spans.push(Span::raw("  "));
                continue;
            }
            let lvl = level(app.day_value(date), max);
            spans.push(cell_span(mode, lvl, date == app.cursor));
        }
        lines.push(Line::from(spans));
    }
    lines.push(legend_line(mode));

    f.render_widget(Paragraph::new(lines), inner);
}

fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let date = app.cursor;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", date.format("%a, %b %e")));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let total = app.day_value(date);
    let sessions = app.day_sessions(date);
    let header = format!(
        "{}   {} session{}",
        fmt_value(app.metric, total),
        sessions,
        if sessions == 1 { "" } else { "s" }
    );

    let by_proj = app.day_by_project(date);
    let by_model = app.day_by_model(date);
    let rows = by_proj.len().max(by_model.len()).min(4);

    let mut lines: Vec<Line> = vec![
        Line::from(header),
        Line::from(format!("{:<28}{}", "by project", "by model")),
    ];
    for i in 0..rows {
        let p = by_proj
            .get(i)
            .map(|b| format!("{:<14}{:>10}", trunc(&b.label, 13), fmt_value(app.metric, b.value)))
            .unwrap_or_default();
        let m = by_model
            .get(i)
            .map(|b| format!("{:<16}{:>10}", trunc(&b.label, 15), fmt_value(app.metric, b.value)))
            .unwrap_or_default();
        lines.push(Line::from(format!("{p:<28}{m}")));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" [m]etric [r]ange [p]roject [M]odel  ←↑↓→ move  [t]oday  [q]uit ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let st = pulsegraph_core::streaks(&app.view, app.today);
    let tot = pulsegraph_core::totals(&app.view, app.metric);
    let line = format!(
        "Total {}  Best {}  Avg/day {}  Active {}  Streak {}  Longest {}",
        fmt_value(app.metric, tot.total),
        tot.best_day.map(|(_, v)| fmt_value(app.metric, v)).unwrap_or_else(|| "—".into()),
        fmt_value(app.metric, tot.avg_per_active_day),
        tot.active_days,
        st.current,
        st.longest,
    );
    f.render_widget(Paragraph::new(line), inner);
}

fn centered(w: u16, h: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect { x, y, width: w.min(area.width), height: h.min(area.height) }
}

fn draw_picker(f: &mut Frame, p: &Picker, area: Rect) {
    let w = 40u16.min(area.width.saturating_sub(4)).max(10);
    let h = ((p.items.len() as u16) + 2)
        .min(area.height.saturating_sub(4))
        .max(3);
    let rect = centered(w, h, area);

    f.render_widget(Clear, rect);
    let title = match p.kind {
        PickerKind::Project => " Project ",
        PickerKind::Model => " Model ",
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let max_rows = inner.height as usize;
    let start = p.selected.saturating_sub(max_rows.saturating_sub(1));
    let mut lines: Vec<Line> = Vec::new();
    for (i, item) in p.items.iter().enumerate().skip(start).take(max_rows) {
        let style = if i == p.selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(format!(" {item}"), style)));
    }
    f.render_widget(Paragraph::new(lines), inner);
}

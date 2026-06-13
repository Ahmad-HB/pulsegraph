pub mod app;
pub mod color;
pub mod ui;

use std::io::{self, Stdout};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::tui::app::{App, PickerKind};
use crate::tui::color::ColorMode;

type Term = Terminal<CrosstermBackend<Stdout>>;

/// Enter the alternate screen, run the loop, and always restore the terminal.
pub fn run(mut app: App) -> io::Result<()> {
    let mode = ColorMode::detect();
    let mut term = setup()?;

    // Restore the terminal even if a panic unwinds through the loop.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let res = run_loop(&mut term, &mut app, mode);
    let restore_res = restore(&mut term);
    // Prefer the loop's error; surface the restore error only if the loop succeeded.
    res.and(restore_res)
}

fn setup() -> io::Result<Term> {
    enable_raw_mode()?;
    let mut out = io::stdout();
    if let Err(e) = execute!(out, EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(e);
    }
    Terminal::new(CrosstermBackend::new(out))
}

fn restore(term: &mut Term) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    term.show_cursor()?;
    Ok(())
}

fn run_loop(term: &mut Term, app: &mut App, mode: ColorMode) -> io::Result<()> {
    loop {
        term.draw(|f| ui::draw(f, app, mode))?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            handle_key(app, key.code, key.modifiers);
            if app.should_quit {
                return Ok(());
            }
        }
    }
}

fn handle_key(app: &mut App, code: KeyCode, mods: KeyModifiers) {
    if app.picker.is_some() {
        handle_picker_key(app, code);
        return;
    }
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => app.should_quit = true,
        KeyCode::Char('m') => app.cycle_metric(),
        KeyCode::Char('r') => app.cycle_range(),
        KeyCode::Char('t') => app.jump_today(),
        KeyCode::Char('p') => app.open_picker(PickerKind::Project),
        KeyCode::Char('M') => app.open_picker(PickerKind::Model),
        KeyCode::Left => app.move_cursor(-1, 0),
        KeyCode::Right => app.move_cursor(1, 0),
        KeyCode::Up => app.move_cursor(0, -1),
        KeyCode::Down => app.move_cursor(0, 1),
        _ => {}
    }
}

fn handle_picker_key(app: &mut App, code: KeyCode) {
    let Some(p) = app.picker.as_mut() else {
        return;
    };
    match code {
        KeyCode::Esc => app.picker = None,
        KeyCode::Up => {
            if p.selected > 0 {
                p.selected -= 1;
            }
        }
        KeyCode::Down => {
            if p.selected + 1 < p.items.len() {
                p.selected += 1;
            }
        }
        KeyCode::Enter => app.apply_picker(),
        _ => {}
    }
}

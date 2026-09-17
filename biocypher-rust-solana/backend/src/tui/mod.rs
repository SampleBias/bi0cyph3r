//! Ratatui terminal workbench. No server, browser, or network is started.
mod app;
mod editor;
mod home;
mod theme;
mod ui;

pub use theme::Theme;

use std::{
    io::{self, IsTerminal},
    time::Duration,
};

use crossterm::{
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind},
    execute,
};

pub fn run() -> anyhow::Result<()> {
    run_with_theme(Theme::default())
}

pub fn run_with_theme(theme: Theme) -> anyhow::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        anyhow::bail!("The TUI needs an interactive terminal. Use --help for scriptable commands.");
    }
    ratatui::run(|terminal| -> anyhow::Result<()> {
        // RAII cleanup also covers errors and unwinding; Ratatui restores raw mode.
        struct PasteGuard;
        impl Drop for PasteGuard {
            fn drop(&mut self) {
                let _ = execute!(io::stdout(), DisableBracketedPaste);
            }
        }
        let _paste = PasteGuard;
        execute!(io::stdout(), EnableBracketedPaste)?;
        let mut app = app::App::default();
        app.theme = theme;
        while !app.quit {
            app.poll();
            terminal.draw(|frame| ui::draw(frame, &mut app))?;
            let frame_time =
                if app.animate && matches!(app.screen, app::Screen::Home | app::Screen::Themes) {
                    33
                } else {
                    80
                };
            if event::poll(Duration::from_millis(frame_time))? {
                match event::read()? {
                    Event::Key(key) if key.kind != KeyEventKind::Release => app.key(key),
                    Event::Paste(text) => {
                        app.paste(&text.replace("\r\n", "\n").replace('\r', "\n"))
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })
}

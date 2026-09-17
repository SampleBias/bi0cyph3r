//! Ratatui terminal workbench. No server, browser, or network is started.
mod app;
mod editor;
mod ui;

use std::{
    io::{self, IsTerminal},
    time::Duration,
};

use crossterm::{
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind},
    execute,
};

pub fn run() -> anyhow::Result<()> {
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
        while !app.quit {
            app.poll();
            terminal.draw(|frame| ui::draw(frame, &mut app))?;
            if event::poll(Duration::from_millis(80))? {
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

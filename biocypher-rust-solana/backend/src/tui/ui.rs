use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Gauge, Paragraph, Wrap},
    Frame,
};

use super::{
    app::{App, Dialog, Field},
    editor::Editor,
};
use crate::workbench::{Operation, MODES};

const BG: Color = Color::Rgb(13, 17, 26);
const PANEL: Color = Color::Rgb(18, 24, 36);
const BORDER: Color = Color::Rgb(46, 59, 78);
const TEXT: Color = Color::Rgb(222, 230, 240);
const MUTED: Color = Color::Rgb(133, 150, 174);
const CYAN: Color = Color::Rgb(87, 222, 218);
const PURPLE: Color = Color::Rgb(180, 155, 255);
const GREEN: Color = Color::Rgb(143, 219, 159);
const GOLD: Color = Color::Rgb(242, 198, 121);
const RED: Color = Color::Rgb(243, 132, 148);

fn panel(title: impl Into<Line<'static>>, active: bool) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if active { CYAN } else { BORDER }))
        .style(Style::default().bg(PANEL).fg(TEXT))
}

fn label(text: impl Into<String>, color: Color) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(color))
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(BG).fg(TEXT)),
        area,
    );
    if area.width < 64 || area.height < 22 {
        frame.render_widget(Paragraph::new("bi0cyph3r\n\nResize to at least 64 × 22.\nYour session is preserved.\n\nCtrl+C to quit.")
            .style(Style::default().fg(CYAN)).wrap(Wrap { trim: false }), area.inner(Margin::new(2, 1)));
        return;
    }
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .split(area.inner(Margin::new(1, 0)));
    header(frame, rows[0], app);
    let body = if area.width >= 108 {
        let columns = Layout::horizontal([
            Constraint::Length(23),
            Constraint::Length(2),
            Constraint::Min(1),
        ])
        .split(rows[1]);
        sidebar(frame, columns[0], app);
        columns[2]
    } else {
        rows[1]
    };
    workspace(frame, body, app);
    let status = Line::from(vec![
        label(
            if app.error { "  !  " } else { "  ›  " },
            if app.error { RED } else { CYAN },
        ),
        label(app.status.clone(), if app.error { RED } else { MUTED }),
    ]);
    frame.render_widget(Paragraph::new(status).wrap(Wrap { trim: false }), rows[2]);
    let hint = if app.editing {
        " Esc done   Tab next field   Ctrl+U clear   Ctrl+R run   Ctrl+S export"
    } else if area.width < 95 {
        " i edit  m mode  ^R run  ^O open  ^S save  1–4 tabs  ? help  q quit"
    } else {
        " i edit   Tab focus   m mode   Ctrl+R run   Ctrl+O import   Ctrl+S export   ? help   q quit"
    };
    frame.render_widget(
        Paragraph::new(hint).style(Style::default().bg(BORDER).fg(TEXT)),
        rows[3],
    );
    if app.dialog.is_some() {
        dialog(frame, app);
    }
}

fn header(frame: &mut Frame, area: Rect, app: &App) {
    let columns = Layout::horizontal([Constraint::Min(1), Constraint::Length(25)]).split(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                label(" ◈ ", CYAN),
                Span::styled(
                    "bi0cyph3r",
                    Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
                ),
                label("  /  DNA WORKBENCH", MUTED),
            ]),
            Line::from(label("    Encode ideas. Read the sequence.", MUTED)),
        ]),
        columns[0],
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(label("● LOCAL  /  OFFLINE", GREEN)),
            Line::from(label(
                format!(
                    "v{}  ·  {} completed",
                    env!("CARGO_PKG_VERSION"),
                    app.completed
                ),
                MUTED,
            )),
        ])
        .alignment(ratatui::layout::Alignment::Right),
        columns[1],
    );
}

fn sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let block = panel(" WORKSPACES ", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let mut lines = vec![Line::raw("")];
    for (i, operation) in Operation::ALL.iter().enumerate() {
        let selected = i == app.page;
        lines.push(
            Line::from(vec![
                label(if selected { " ▌ " } else { "   " }, CYAN),
                label(
                    format!("{}  {:<10}", i + 1, operation.title()),
                    if selected { CYAN } else { TEXT },
                ),
                label(
                    if app.pages[i].output.is_some() {
                        "●"
                    } else {
                        "·"
                    },
                    if selected { CYAN } else { MUTED },
                ),
            ])
            .style(if selected {
                Style::default().bg(BORDER)
            } else {
                Style::default()
            }),
        );
        lines.push(Line::raw(""));
    }
    lines.extend([
        Line::from(label("   SESSION", MUTED)),
        Line::raw(""),
        Line::from(label("   Files stay local", GREEN)),
        Line::from(label("   No network needed", MUTED)),
        Line::raw(""),
    ]);
    if inner.height >= 24 {
        lines.push(Line::from(label("   RECENT ACTIVITY", MUTED)));
        lines.push(Line::raw(""));
        if app.activity.is_empty() {
            lines.push(Line::from(label("   A fresh strand…", MUTED)));
        }
        for entry in &app.activity {
            lines.push(Line::from(label(format!("   {entry}"), MUTED)));
        }
    }
    frame.render_widget(Paragraph::new(lines), inner);
    if inner.height > 28 {
        let helix = Rect::new(
            inner.x + 3,
            inner.bottom() - 6,
            inner.width.saturating_sub(6),
            5,
        );
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    label("A", GREEN),
                    label(" ──── ", BORDER),
                    label("T", GOLD),
                ]),
                Line::from(vec![
                    label("  C", CYAN),
                    label(" ─ ", BORDER),
                    label("G", PURPLE),
                ]),
                Line::from(vec![label("   ╳", MUTED)]),
                Line::from(vec![
                    label("  G", PURPLE),
                    label(" ─ ", BORDER),
                    label("C", CYAN),
                ]),
                Line::from(vec![
                    label("T", GOLD),
                    label(" ──── ", BORDER),
                    label("A", GREEN),
                ]),
            ]),
            helix,
        );
    }
}

fn workspace(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(4),
        Constraint::Min(1),
    ])
    .split(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    format!("{:02} / {}", app.page + 1, app.operation().title()),
                    Style::default().fg(CYAN).bold(),
                ),
                label(
                    if app.editing {
                        "   INSERT"
                    } else {
                        "   NORMAL"
                    },
                    PURPLE,
                ),
            ]),
            Line::from(label(app.operation().description(), MUTED)),
        ]),
        rows[0],
    );
    metrics(frame, rows[1], app);
    let panes =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).split(rows[2]);
    input(frame, panes[0], app);
    output(frame, panes[1], app);
}

fn metrics(frame: &mut Frame, area: Rect, app: &App) {
    let columns = Layout::horizontal([Constraint::Percentage(25); 4]).split(area);
    let stats = app.workspace().output.as_ref().map(|o| &o.stats);
    let values = [
        (
            " LENGTH ",
            stats.map_or("—".into(), |s| format!("{} bp", s.length)),
            CYAN,
        ),
        (
            " GC CONTENT ",
            stats.map_or("—".into(), |s| format!("{:.1}%", s.gc_content)),
            GREEN,
        ),
        (
            " MODE ",
            if app.operation() == Operation::Safety {
                "screen".into()
            } else {
                app.workspace().mode.to_string()
            },
            PURPLE,
        ),
        (
            " SESSION ",
            if app.job.is_some() {
                "working".into()
            } else {
                "local".into()
            },
            GOLD,
        ),
    ];
    for (i, (title, value, color)) in values.into_iter().enumerate() {
        frame.render_widget(
            Paragraph::new(format!(" {value}"))
                .style(Style::default().fg(color).bold())
                .block(panel(title, false)),
            columns[i],
        );
    }
}

fn input(frame: &mut Frame, area: Rect, app: &App) {
    let workspace = app.workspace();
    let extra_fields: Vec<Field> = app
        .fields()
        .into_iter()
        .filter(|f| !matches!(f, Field::Input | Field::Output | Field::Mode))
        .collect();
    let has_mode = app.operation() != Operation::Safety;
    let compact = area.height < 6 + extra_fields.len() as u16 * 3;
    let mut constraints = vec![Constraint::Min(3)];
    if has_mode {
        constraints.push(Constraint::Length(if compact { 1 } else { 3 }));
    }
    constraints.extend(
        extra_fields
            .iter()
            .map(|_| Constraint::Length(if compact { 2 } else { 3 })),
    );
    let rows = Layout::vertical(constraints).split(area);
    let title = if matches!(app.operation(), Operation::Decode | Operation::Safety) {
        " INPUT / DNA or FASTA "
    } else {
        " INPUT / message "
    };
    editor(
        frame,
        rows[0],
        &workspace.input,
        title,
        app.focus == Field::Input,
        app.editing && app.focus == Field::Input,
        false,
        "Press i to type here.\n\nCtrl+O  import a text file\nCtrl+R  run operation",
    );
    let mut next = 1;
    if has_mode {
        let spans = if area.width >= 46 {
            MODES
                .iter()
                .map(|mode| {
                    Span::styled(
                        format!(" {mode} "),
                        if *mode == workspace.mode {
                            Style::default().fg(BG).bg(PURPLE).bold()
                        } else {
                            Style::default().fg(MUTED)
                        },
                    )
                })
                .collect::<Vec<_>>()
        } else {
            vec![label(format!(" ‹ {} ›   m cycle", workspace.mode), PURPLE)]
        };
        let mode = Paragraph::new(Line::from(spans));
        frame.render_widget(
            if compact {
                mode
            } else {
                mode.block(panel(" MODE ", app.focus == Field::Mode))
            },
            rows[next],
        );
        next += 1;
    }
    for field in extra_fields {
        if field == Field::Structure {
            let block = panel(" STRUCTURE / x cycle ", app.focus == field);
            let block = if compact {
                block.borders(Borders::TOP)
            } else {
                block
            };
            frame.render_widget(
                Paragraph::new(workspace.structure.label())
                    .style(Style::default().fg(PURPLE))
                    .block(block),
                rows[next],
            );
            next += 1;
            continue;
        }
        let (value, title, secret) = match field {
            Field::Password => (&workspace.password, " PASSWORD ", true),
            Field::K1 => (&workspace.k1, " KEY 1 / base64 ", true),
            Field::K2 => (&workspace.k2, " KEY 2 / base64 ", true),
            Field::Name => (&workspace.name, " SEQUENCE NAME ", false),
            _ => unreachable!(),
        };
        editor(
            frame,
            rows[next],
            value,
            title,
            app.focus == field,
            app.editing && app.focus == field,
            secret,
            "Enter to edit",
        );
        next += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn editor(
    frame: &mut Frame,
    area: Rect,
    value: &Editor,
    title: &'static str,
    focused: bool,
    editing: bool,
    secret: bool,
    placeholder: &str,
) {
    let block = panel(title, focused);
    let block = if area.height <= 2 {
        block.borders(Borders::TOP)
    } else {
        block
    };
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.is_empty() {
        return;
    }
    if value.text.is_empty() && !editing {
        frame.render_widget(
            Paragraph::new(placeholder)
                .style(Style::default().fg(MUTED))
                .wrap(Wrap { trim: false }),
            inner,
        );
        return;
    }
    let (row, column) = value.position(secret);
    let scroll_y = row.saturating_sub(inner.height.saturating_sub(1) as usize);
    let scroll_x = column.saturating_sub(inner.width.saturating_sub(1) as usize);
    frame.render_widget(
        Paragraph::new(value.viewport(
            secret,
            scroll_y,
            scroll_x,
            inner.width as usize,
            inner.height as usize,
        )),
        inner,
    );
    if editing {
        frame.set_cursor_position((
            inner.x + (column - scroll_x) as u16,
            inner.y + (row - scroll_y) as u16,
        ));
    }
}

fn base_color(base: char) -> Color {
    match base {
        'A' => GREEN,
        'T' => GOLD,
        'C' => CYAN,
        'G' => PURPLE,
        _ => TEXT,
    }
}

fn output(frame: &mut Frame, area: Rect, app: &mut App) {
    let active_job = app.job.as_ref().is_some_and(|j| j.page == app.page);
    let title = if active_job {
        format!(" {} PROCESSING ", ["◐", "◓", "◑", "◒"][app.tick % 4])
    } else {
        " RESULT / Ctrl+S export ".into()
    };
    let block = panel(title, app.focus == Field::Output);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.is_empty() {
        return;
    }
    let Some(result) = &app.workspace().output else {
        let rows = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(8),
            Constraint::Min(1),
        ])
        .split(inner);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(label("A  ·  T  ·  C  ·  G", CYAN)),
                Line::raw(""),
                Line::from(label("Your next strand starts here.", TEXT)),
                Line::raw(""),
                Line::from(label("Write or import your input,", MUTED)),
                Line::from(label("choose a mode, then Ctrl+R.", MUTED)),
            ])
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: false }),
            rows[1],
        );
        return;
    };
    let rows = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(inner);
    let is_dna = matches!(result.operation, Operation::Encode | Operation::Plasmid);
    let has_keys = result.keys.is_some();
    let counts = &result.stats.bases;
    let legend = Line::from(vec![
        label(format!(" A:{} ", counts.a), GREEN),
        label(format!("T:{} ", counts.t), GOLD),
        label(format!("C:{} ", counts.c), CYAN),
        label(format!("G:{}", counts.g), PURPLE),
    ]);
    let gc_ratio = result.stats.gc_content / 100.0;
    if is_dna {
        let width = rows[0].width.saturating_sub(9).max(4) as usize;
        let total = result.sequence.len().div_ceil(width);
        let max_scroll = total
            .saturating_sub(rows[0].height as usize)
            .min(u16::MAX as usize) as u16;
        let scroll = app.workspace().scroll.min(max_scroll);
        let lines: Vec<Line> = result
            .sequence
            .as_bytes()
            .chunks(width)
            .enumerate()
            .skip(scroll as usize)
            .take(rows[0].height as usize)
            .map(|(i, chunk)| {
                let mut spans = vec![label(format!("{:>6}  ", i * width + 1), MUTED)];
                spans.extend(
                    chunk
                        .iter()
                        .map(|b| label((*b as char).to_string(), base_color(*b as char))),
                );
                Line::from(spans)
            })
            .collect();
        frame.render_widget(Paragraph::new(lines), rows[0]);
        app.pages[app.page].scroll = scroll;
    } else {
        // Render control characters as visible escape sequences, never terminal commands.
        let safe: String = result
            .text
            .chars()
            .flat_map(|c| {
                if c.is_control() && c != '\n' && c != '\t' {
                    c.escape_default().collect::<Vec<_>>()
                } else if c == '\t' {
                    vec![' '; 4]
                } else {
                    vec![c]
                }
            })
            .collect();
        let paragraph = Paragraph::new(safe).wrap(Wrap { trim: false });
        let max_scroll = paragraph
            .line_count(rows[0].width)
            .saturating_sub(rows[0].height as usize)
            .min(u16::MAX as usize) as u16;
        let scroll = app.workspace().scroll.min(max_scroll);
        frame.render_widget(paragraph.scroll((scroll, 0)), rows[0]);
        app.pages[app.page].scroll = scroll;
    }
    let bottom = Layout::vertical([Constraint::Length(1); 3]).split(rows[1]);
    frame.render_widget(Paragraph::new(legend), bottom[0]);
    frame.render_widget(
        Gauge::default()
            .ratio(gc_ratio.clamp(0.0, 1.0))
            .gauge_style(Style::default().fg(CYAN).bg(BORDER))
            .label(format!("GC {:.1}%", gc_ratio * 100.0)),
        bottom[1],
    );
    frame.render_widget(
        Paragraph::new(if has_keys {
            " Ctrl+K save keys · v reveal"
        } else {
            " d → decode   s → safety   PgDn ↓"
        })
        .style(Style::default().fg(if has_keys { GOLD } else { MUTED })),
        bottom[2],
    );
}

fn dialog(frame: &mut Frame, app: &App) {
    let Some(dialog) = &app.dialog else {
        return;
    };
    let (title, height) = match dialog {
        Dialog::Help => (" KEYBOARD / FIELD GUIDE ", 23),
        Dialog::Import(_) => (" IMPORT / local text file ", 12),
        Dialog::Export { keys: true, .. } => (" EXPORT / secret key bundle ", 12),
        Dialog::Export { .. } => (" EXPORT / result ", 12),
        Dialog::Keys => (" SPLIT KEYS / keep separately ", 12),
        Dialog::ConfirmQuit => (" LEAVE SESSION? ", 8),
        Dialog::ConfirmRun => (" REPLACE UNSAVED KEYS? ", 8),
    };
    let width = frame.area().width.saturating_sub(6).min(76);
    let height = height.min(frame.area().height.saturating_sub(2));
    let area = Rect::new(
        (frame.area().width - width) / 2,
        (frame.area().height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, area);
    let block = panel(title, true);
    let inner = block.inner(area).inner(Margin::new(1, 0));
    frame.render_widget(block, area);
    match dialog {
        Dialog::Help => {
            let help = vec![
                Line::from(label("MOVE", CYAN)),
                Line::raw("1–4 / F1–F4     Switch workspaces (F keys also while editing)"),
                Line::raw("Tab / Shift+Tab  Focus next / previous field"),
                Line::raw("i / Enter       Edit the focused field"),
                Line::raw("Esc             Leave editing / close a dialog"),
                Line::raw("m / ← →         Cycle encoding mode"),
                Line::raw("x               Cycle plasmid structure"),
                Line::raw(""),
                Line::from(label("WORK", CYAN)),
                Line::raw("Ctrl+R / F5     Run the current operation"),
                Line::raw("Ctrl+O          Import a UTF-8 text or single-record FASTA file"),
                Line::raw("Ctrl+S / F6     Export result; Tab changes the file format"),
                Line::raw("Ctrl+K / v      Export / reveal split keys"),
                Line::raw("d / s           Send result to Decode / Safety"),
                Line::raw("PgUp / PgDn     Scroll result (j/k or arrows with result focus)"),
                Line::raw("Ctrl+U          Clear the field being edited"),
                Line::raw(""),
                Line::from(label("SESSION", CYAN)),
                Line::raw("q / Ctrl+Q / F10 Quit (Ctrl+C also works while editing)"),
                Line::raw("Session contents stay in memory until explicitly exported."),
                Line::from(label("Any key closes this guide.", MUTED)),
            ];
            frame.render_widget(Paragraph::new(help).wrap(Wrap { trim: false }), inner);
        }
        Dialog::Import(path) | Dialog::Export { path, .. } => {
            let rows = Layout::vertical([
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(inner);
            let description = match dialog {
                Dialog::Import(_) => "UTF-8 text / single FASTA record · up to 256 KiB".into(),
                Dialog::Export { keys: true, .. } => {
                    "Contains BOTH secret keys. Store them separately after export.".into()
                }
                Dialog::Export { format, .. } => format!(
                    "Format: {}    [Tab] change format",
                    format.extension().to_uppercase()
                ),
                _ => unreachable!(),
            };
            frame.render_widget(
                Paragraph::new(description)
                    .style(Style::default().fg(GOLD))
                    .wrap(Wrap { trim: false }),
                rows[0],
            );
            editor(frame, rows[1], path, " PATH ", true, true, false, "");
            let note = if matches!(dialog, Dialog::Import(_)) {
                "Enter import · Esc cancel\nUse a relative or absolute path."
            } else {
                "Enter save · Esc cancel\nExisting files are never overwritten."
            };
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(label(note.lines().next().unwrap_or(""), MUTED)),
                    Line::from(label(note.lines().nth(1).unwrap_or(""), MUTED)),
                    Line::from(label(if app.error { app.status.as_str() } else { "" }, RED)),
                ])
                .wrap(Wrap { trim: false }),
                rows[2],
            );
        }
        Dialog::Keys => {
            if let Some(keys) = app
                .workspace()
                .output
                .as_ref()
                .and_then(|o| o.keys.as_ref())
            {
                frame.render_widget(
                    Paragraph::new(Text::from(vec![
                        Line::from(label("K1 / user key", CYAN)),
                        Line::raw(keys.k1_base64.clone()),
                        Line::raw(""),
                        Line::from(label("K2 / second key", PURPLE)),
                        Line::raw(keys.k2_base64.clone()),
                        Line::raw(""),
                        Line::from(label(
                            "Both keys are needed to decode. Ctrl+K exports a key bundle.",
                            GOLD,
                        )),
                        Line::from(label("Any key closes this view.", MUTED)),
                    ]))
                    .wrap(Wrap { trim: false }),
                    inner,
                );
            }
        }
        Dialog::ConfirmQuit | Dialog::ConfirmRun => {
            let message = if matches!(dialog, Dialog::ConfirmQuit) {
                "You have unexported split keys or a running operation.\nLeaving discards this session.\n\n[y] discard and quit   [n / Esc] return"
            } else {
                "This result contains unexported split keys.\nExport them with Ctrl+K before replacing the result.\n\n[y] replace result   [n / Esc] return"
            };
            frame.render_widget(
                Paragraph::new(message)
                    .style(Style::default().fg(GOLD))
                    .wrap(Wrap { trim: false }),
                inner,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dna::EncodingMode;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn layouts_render_at_common_sizes_and_mask_secrets() {
        for (width, height) in [(140, 42), (108, 32), (80, 24), (64, 22), (40, 12)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            let mut app = App::default();
            app.pages[0].mode = EncodingMode::Secure;
            app.pages[0].password = Editor::new("hidden-test-password");
            terminal.draw(|f| draw(f, &mut app)).unwrap();
            let rendered: String = terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect();
            assert!(!rendered.contains("hidden-test-password"));
            assert!(rendered.contains("bi0cyph3r"));
            if width >= 64 {
                assert!(rendered.contains("Ctrl+R"));
            }
            app.dialog = Some(Dialog::Help);
            terminal.draw(|f| draw(f, &mut app)).unwrap();
        }
    }
}

//! Startup, field guide, and theme preview screens. All artwork is terminal-native.

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, Wrap},
    Frame,
};

use super::{
    app::{App, Screen},
    theme::{Palette, Theme},
};

pub const LOGO: [&str; 3] = [
    "░█▀▄░▀█▀░▄▀▄░█▀▀░█░█░█▀█░█░█░▀▀█░█▀▄",
    "░█▀▄░░█░░█/█░█░░░░█░░█▀▀░█▀█░░▀▄░█▀▄",
    "░▀▀░░▀▀▀░░▀░░▀▀▀░░▀░░▀░░░▀░▀░▀▀░░▀░▀",
];

fn span(text: impl Into<String>, color: Color) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(color))
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    )
}

fn panel(title: &'static str, colors: Palette) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(colors.text).bg(colors.panel))
        .border_style(Style::default().fg(colors.border))
}

pub fn draw(frame: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::Home => landing(frame, app),
        Screen::Guide => guide(frame, app),
        Screen::Themes => themes(frame, app),
        Screen::Workbench => {}
    }
}

fn landing(frame: &mut Frame, app: &App) {
    let colors = app.theme.palette();
    let area = frame.area().inner(Margin::new(2, 1));
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(6),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);
    let top = Layout::horizontal([Constraint::Percentage(50); 2]).split(rows[0]);
    frame.render_widget(
        Paragraph::new("◈ TERMINAL DNA LAB").style(Style::default().fg(colors.muted)),
        top[0],
    );
    frame.render_widget(
        Paragraph::new(format!(
            "{} / v{}",
            app.theme.name(),
            env!("CARGO_PKG_VERSION")
        ))
        .style(Style::default().fg(colors.muted))
        .alignment(Alignment::Right),
        top[1],
    );

    let logo: Vec<Line> = LOGO
        .iter()
        .enumerate()
        .map(|(i, row)| {
            Line::from(
                row.chars()
                    .map(|ch| {
                        span(
                            ch.to_string(),
                            if ch == '░' {
                                colors.border
                            } else if i == 1 {
                                colors.text
                            } else if i == 2 {
                                colors.secondary
                            } else {
                                colors.accent
                            },
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    frame.render_widget(Paragraph::new(logo).alignment(Alignment::Center), rows[2]);
    frame.render_widget(
        Paragraph::new("DNA cryptography • all in your terminal")
            .style(Style::default().fg(colors.muted))
            .alignment(Alignment::Center),
        rows[3],
    );
    frame.render_widget(
        Helix {
            phase: app.rotation,
            colors,
        },
        centered(rows[4], 54, 23),
    );

    let menu = Layout::horizontal([
        Constraint::Percentage(34),
        Constraint::Percentage(33),
        Constraint::Percentage(33),
    ])
    .split(centered(rows[5], 68, 3));
    for (i, title) in ["[s] start", "[g] guide", "[t] themes"]
        .into_iter()
        .enumerate()
    {
        let selected = app.menu_index == i;
        let style = if selected {
            Style::default().fg(colors.bg).bg(colors.accent).bold()
        } else {
            Style::default().fg(colors.text).bg(colors.panel)
        };
        frame.render_widget(
            Paragraph::new(title)
                .alignment(Alignment::Center)
                .style(style)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(if selected {
                            colors.accent
                        } else {
                            colors.border
                        })),
                ),
            menu[i],
        );
    }
    frame.render_widget(
        Paragraph::new("s - start   g - guide   t - themes")
            .style(Style::default().fg(colors.muted))
            .alignment(Alignment::Center),
        rows[6],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            span("Made with ASCII Heart", colors.muted),
            span(" by ", colors.muted),
            Span::styled("S4MPL3BI4S", Style::default().fg(colors.text).bold()),
            span(" <3", colors.secondary),
        ]))
        .alignment(Alignment::Center),
        rows[8],
    );
    frame.render_widget(
        Paragraph::new(if app.animate {
            "← → select / Enter open / Space pause / q quit"
        } else {
            "← → select / Enter open / Space spin / q quit"
        })
        .style(Style::default().fg(colors.muted))
        .alignment(Alignment::Center),
        rows[9],
    );
}

/// Two strands projected around a vertical axis. Depth controls perspective,
/// draw order, and brightness; phase is elapsed-time based rather than key ticks.
struct Helix {
    phase: f64,
    colors: Palette,
}

impl Widget for Helix {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        if area.width < 8 || area.height < 3 {
            return;
        }
        // A Braille cell has 2 × 4 dots. Sampling at dot resolution keeps the
        // rotating backbones smooth while the base pairs remain ordinary text.
        let pixel_width = area.width as usize * 2;
        let pixel_height = area.height as usize * 4;
        let center = (pixel_width - 1) as f64 / 2.0;
        let radius = ((pixel_width as f64 - 8.0) * 0.32).min(34.0);
        let turns = if area.height < 14 { 1.05 } else { 1.65 };
        let mut nodes = Vec::with_capacity(pixel_height);
        for y in 0..pixel_height {
            let angle =
                y as f64 / (pixel_height - 1) as f64 * std::f64::consts::TAU * turns + self.phase;
            let depth = angle.cos();
            let x1 = (center + radius * angle.sin() / (1.0 - depth * 0.16)).round() as usize;
            let x2 = (center - radius * angle.sin() / (1.0 + depth * 0.16)).round() as usize;
            nodes.push((x1, x2, depth));
        }
        let put = |buffer: &mut Buffer, x: usize, y: usize, symbol: &str, color: Color| {
            if x < area.width as usize && y < area.height as usize {
                buffer[(area.x + x as u16, area.y + y as u16)]
                    .set_symbol(symbol)
                    .set_fg(color);
            }
        };
        for row in (0..area.height as usize).step_by(2) {
            let (x1, x2, _) = nodes[row * 4 + 2];
            let (x1, x2) = (x1 / 2, x2 / 2);
            put(buffer, center as usize / 2, row, ".", self.colors.border);
            if x1.abs_diff(x2) > 4 {
                for x in x1.min(x2) + 1..x1.max(x2) {
                    put(buffer, x, row, "-", self.colors.border);
                }
                let pair = if row % 4 == 0 { ("A", "T") } else { ("C", "G") };
                let a = (x1 as f64 * 0.72 + x2 as f64 * 0.28).round() as usize;
                let b = (x1 as f64 * 0.28 + x2 as f64 * 0.72).round() as usize;
                put(buffer, a, row, pair.0, self.colors.positive);
                put(buffer, b, row, pair.1, self.colors.warning);
            }
        }
        let mut cells = vec![
            (0u8, self.colors.muted, f64::NEG_INFINITY);
            area.width as usize * area.height as usize
        ];
        let dots = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
        for (y, &(x1, x2, depth)) in nodes.iter().enumerate() {
            let previous = nodes[y.saturating_sub(1)];
            for (x, last, z, front_color) in [
                (x1, previous.0, depth, self.colors.accent),
                (x2, previous.1, -depth, self.colors.secondary),
            ] {
                for pixel_x in last.min(x)..=last.max(x) {
                    if pixel_x >= pixel_width {
                        continue;
                    }
                    let cell = &mut cells[(y / 4) * area.width as usize + pixel_x / 2];
                    cell.0 |= dots[y % 4][pixel_x % 2];
                    if z > cell.2 {
                        cell.1 = if z >= 0.0 {
                            front_color
                        } else {
                            self.colors.muted
                        };
                        cell.2 = z;
                    }
                }
            }
        }
        for (i, (mask, color, _)) in cells.into_iter().enumerate() {
            if mask != 0 {
                let symbol = char::from_u32(0x2800 + mask as u32).unwrap();
                put(
                    buffer,
                    i % area.width as usize,
                    i / area.width as usize,
                    &symbol.to_string(),
                    color,
                );
            }
        }
    }
}

fn guide(frame: &mut Frame, app: &mut App) {
    let colors = app.theme.palette();
    let area = centered(
        frame.area().inner(Margin::new(2, 1)),
        94,
        frame.area().height,
    );
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(area);
    frame.render_widget(
        Paragraph::new("FIELD GUIDE / bi0cyph3r").style(Style::default().fg(colors.accent).bold()),
        rows[0],
    );
    let section = |text: &str| {
        Line::from(Span::styled(
            text.to_owned(),
            Style::default().fg(colors.accent).bold(),
        ))
    };
    let text = vec![
        section("YOUR FIRST STRAND"),
        Line::raw("1. Press s to start the workbench."),
        Line::raw("2. Press i, write a message, then Ctrl+R to encode it."),
        Line::raw("3. Press d to send the result to Decode, then Ctrl+R."),
        Line::raw("4. Ctrl+S exports your result. Tab changes the file format."),
        Line::raw(""),
        section("FOUR WORKSPACES"),
        Line::raw("1 / Encode   Text to DNA, with four encoding modes."),
        Line::raw("2 / Decode   DNA or a single FASTA record back to text."),
        Line::raw("3 / Safety   GC content, sequence characteristics, and local signature checks."),
        Line::raw("4 / Plasmid  Named payloads, optional markers, and the existing eGFP cassette."),
        Line::raw(""),
        section("MOVE & EDIT"),
        Line::raw("1–4 / F1–F4   Switch workspace; F keys also work during editing."),
        Line::raw("Tab / Shift+Tab   Move between input, options, credentials, and result."),
        Line::raw("i / Enter   Edit a field. Arrows and Home/End move the cursor."),
        Line::raw("Esc   Finish editing; press Esc again to return to the start menu."),
        Line::raw("Ctrl+U   Clear the field being edited. Paste also works here."),
        Line::raw("m   Cycle encoding mode. x cycles the plasmid structure."),
        Line::raw(""),
        section("RUN & EXPORT"),
        Line::raw("Ctrl+R / F5   Run the current operation."),
        Line::raw("Ctrl+O   Import a UTF-8 text file or single-record FASTA file."),
        Line::raw("Ctrl+S / F6   Export TXT, FASTA, or JSON without overwriting existing files."),
        Line::raw("Ctrl+K / v   Export / reveal generated split keys."),
        Line::raw("d / s   Send the result to Decode / Safety."),
        Line::raw("PgUp / PgDn   Scroll the result."),
        Line::raw(""),
        section("CHOOSE A MODE"),
        Line::raw("Basic   UTF-8 bytes mapped to DNA; no encryption."),
        Line::raw("Nanopore   Triplets, parity, redundancy, and padding."),
        Line::raw("Secure   The existing AES-256-CBC format; enter a password."),
        Line::raw(
            "Split Key   Both K1 and K2 are needed to decode. Export and store them separately.",
        ),
        Line::raw(""),
        section("MAKE IT YOURS"),
        Line::raw("t / F9   Preview themes; Enter applies, Esc cancels."),
        Line::raw("Original / Crimson / Paper / Monochrome / Amber"),
        Line::raw("Space on the start menu pauses or resumes the rotating DNA strand."),
        Line::raw(
            "Launch with --theme paper, or set BIOCYPHER_THEME=paper, for a preferred palette.",
        ),
        Line::raw(""),
        section("YOUR SESSION"),
        Line::raw(
            "Returning home keeps your inputs and results. Export what you need before quitting.",
        ),
        Line::raw("q / Ctrl+Q / F10 quits the workbench; Ctrl+C also works while editing."),
        Line::raw("The safety report is a local heuristic, not a biological safety certification."),
        Line::raw(""),
        Line::from(span(
            "Made with ASCII Heart by S4MPL3BI4S <3",
            colors.secondary,
        )),
    ];
    let block = panel(" GUIDE / scroll to explore ", colors);
    let inner = block.inner(rows[1]).inner(Margin::new(1, 0));
    frame.render_widget(block, rows[1]);
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
    let max_scroll = paragraph
        .line_count(inner.width)
        .saturating_sub(inner.height as usize)
        .min(u16::MAX as usize) as u16;
    app.guide_scroll = app.guide_scroll.min(max_scroll);
    frame.render_widget(paragraph.scroll((app.guide_scroll, 0)), inner);
    frame.render_widget(
        Paragraph::new("↑ ↓ / PgUp PgDn scroll   s start   Esc back")
            .style(Style::default().fg(colors.muted))
            .wrap(Wrap { trim: false }),
        rows[2],
    );
}

fn themes(frame: &mut Frame, app: &App) {
    let colors = app.theme.palette();
    let area = centered(
        frame.area().inner(Margin::new(2, 1)),
        100,
        frame.area().height,
    );
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(2),
    ])
    .split(area);
    frame.render_widget(
        Paragraph::new("THEMES / find your frequency")
            .style(Style::default().fg(colors.accent).bold()),
        rows[0],
    );
    frame.render_widget(
        Paragraph::new("Live preview. Your whole workbench follows this palette.")
            .style(Style::default().fg(colors.muted)),
        rows[1],
    );
    let columns = Layout::horizontal(if area.width >= 72 {
        vec![
            Constraint::Length(34),
            Constraint::Length(2),
            Constraint::Min(1),
        ]
    } else {
        vec![Constraint::Min(1)]
    })
    .split(rows[2]);
    let choices = Layout::vertical([Constraint::Length(3); 5]).split(columns[0]);
    for (i, theme) in Theme::ALL.into_iter().enumerate() {
        let selected = app.theme_index == i;
        let style = if selected {
            Style::default().fg(colors.accent).bg(colors.panel).bold()
        } else {
            Style::default().fg(colors.text).bg(colors.bg)
        };
        let lines = vec![
            Line::from(format!(
                " {} [{}] {}",
                if selected { "▌" } else { " " },
                i + 1,
                theme.name()
            )),
            Line::from(span(
                format!("       {}", theme.description()),
                colors.muted,
            )),
        ];
        frame.render_widget(Paragraph::new(lines).style(style), choices[i]);
    }
    if columns.len() > 1 {
        let block = panel(" PREVIEW / bi0cyph3r ", colors);
        let inner = block.inner(columns[2]).inner(Margin::new(1, 0));
        frame.render_widget(block, columns[2]);
        let preview = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
            Constraint::Length(3),
        ])
        .split(inner);
        frame.render_widget(
            Paragraph::new(app.theme.name())
                .alignment(Alignment::Center)
                .style(Style::default().fg(colors.accent).bold()),
            preview[0],
        );
        frame.render_widget(
            Helix {
                phase: app.rotation,
                colors,
            },
            centered(preview[1], 34, 19),
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                span(" A ", colors.positive),
                span(" T ", colors.warning),
                span(" C ", colors.accent),
                span(" G ", colors.secondary),
            ]))
            .alignment(Alignment::Center),
            preview[2],
        );
        frame.render_widget(
            Paragraph::new("[Enter] apply theme")
                .alignment(Alignment::Center)
                .style(Style::default().fg(colors.bg).bg(colors.accent).bold()),
            centered(preview[3], 26, 1),
        );
    }
    frame.render_widget(
        Paragraph::new("↑ ↓ / 1–5 preview   Enter apply   Esc cancel")
            .style(Style::default().fg(colors.muted))
            .wrap(Wrap { trim: false }),
        rows[3],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helix_rotates_and_clips_to_its_own_area() {
        let outer = Rect::new(0, 0, 50, 30);
        let area = Rect::new(5, 4, 38, 20);
        let mut a = Buffer::empty(outer);
        let mut b = Buffer::empty(outer);
        Helix {
            phase: 0.0,
            colors: Theme::Original.palette(),
        }
        .render(area, &mut a);
        Helix {
            phase: 1.0,
            colors: Theme::Original.palette(),
        }
        .render(area, &mut b);
        assert_ne!(a, b);
        for x in 0..outer.width {
            assert_eq!(a[(x, 0)].symbol(), " ");
            assert_eq!(a[(x, 29)].symbol(), " ");
        }
    }
}

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::workbench::MAX_INPUT_BYTES;

/// Cursor offsets are UTF-8 byte boundaries; screen columns use display width.
#[derive(Clone, Default)]
pub struct Editor {
    pub text: String,
    pub cursor: usize,
}

impl Editor {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            cursor: text.len(),
            text,
        }
    }

    pub fn insert(&mut self, text: &str, multiline: bool) {
        let text: String = text
            .chars()
            .filter(|c| !c.is_control() || (multiline && matches!(c, '\n' | '\t')))
            .collect();
        if self.text.len() + text.len() > MAX_INPUT_BYTES {
            return;
        }
        self.text.insert_str(self.cursor, &text);
        self.cursor += text.len();
    }

    pub fn position(&self, masked: bool) -> (usize, usize) {
        let before = &self.text[..self.cursor];
        let row = before.bytes().filter(|b| *b == b'\n').count();
        let line = before.rsplit('\n').next().unwrap_or("");
        let column = if masked {
            line.chars().count()
        } else {
            line.replace('\t', "    ").width()
        };
        (row, column)
    }

    pub fn display(&self, masked: bool) -> String {
        if masked {
            "•".repeat(self.text.chars().count())
        } else {
            self.text
                .replace('\t', "    ")
                .chars()
                .filter(|c| !c.is_control() || *c == '\n')
                .collect()
        }
    }

    /// Clip with usize coordinates, so long imported DNA lines do not overflow
    /// Ratatui's u16 paragraph scroll offsets.
    pub fn viewport(
        &self,
        masked: bool,
        top: usize,
        left: usize,
        width: usize,
        height: usize,
    ) -> String {
        self.display(masked)
            .split('\n')
            .skip(top)
            .take(height)
            .map(|line| {
                let mut column = 0;
                let mut visible = String::new();
                for ch in line.chars() {
                    let start = column;
                    let size = ch.width().unwrap_or(0);
                    column += size;
                    if column <= left && size > 0 {
                        continue;
                    }
                    if column > left + width {
                        break;
                    }
                    if start < left {
                        visible.push_str(&" ".repeat(column - left));
                    } else {
                        visible.push(ch);
                    }
                }
                visible
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn key(&mut self, key: KeyEvent, multiline: bool) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('u') if ctrl => {
                self.text.clear();
                self.cursor = 0;
            }
            KeyCode::Char(c) if !ctrl && !key.modifiers.contains(KeyModifiers::ALT) => {
                self.insert(&c.to_string(), multiline)
            }
            KeyCode::Enter if multiline => self.insert("\n", true),
            KeyCode::Left => {
                self.cursor = self.text[..self.cursor]
                    .char_indices()
                    .next_back()
                    .map_or(0, |(i, _)| i)
            }
            KeyCode::Right => {
                if let Some(c) = self.text[self.cursor..].chars().next() {
                    self.cursor += c.len_utf8();
                }
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let previous = self.text[..self.cursor]
                        .char_indices()
                        .next_back()
                        .unwrap()
                        .0;
                    self.text.drain(previous..self.cursor);
                    self.cursor = previous;
                }
            }
            KeyCode::Delete => {
                if let Some(c) = self.text[self.cursor..].chars().next() {
                    self.text.drain(self.cursor..self.cursor + c.len_utf8());
                }
            }
            KeyCode::Home => {
                self.cursor = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1)
            }
            KeyCode::End => {
                self.cursor += self.text[self.cursor..]
                    .find('\n')
                    .unwrap_or(self.text.len() - self.cursor)
            }
            KeyCode::Up | KeyCode::Down if multiline => {
                let start = self.text[..self.cursor].rfind('\n').map_or(0, |i| i + 1);
                let column = self.text[start..self.cursor].chars().count();
                let target = if key.code == KeyCode::Up && start > 0 {
                    Some(self.text[..start - 1].rfind('\n').map_or(0, |i| i + 1))
                } else if key.code == KeyCode::Down {
                    self.text[self.cursor..]
                        .find('\n')
                        .map(|i| self.cursor + i + 1)
                } else {
                    None
                };
                if let Some(target) = target {
                    let line = self.text[target..].split('\n').next().unwrap_or("");
                    self.cursor = target
                        + line
                            .char_indices()
                            .nth(column)
                            .map_or(line.len(), |(i, _)| i);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_cursor_editing_and_multiline_navigation() {
        let mut editor = Editor::new("a🧬\né界");
        editor.key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE), true);
        assert_eq!(editor.position(false), (1, 1));
        editor.key(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE), true);
        assert_eq!(editor.text, "a🧬\n界");
        editor.key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), true);
        assert_eq!(editor.cursor, 0);
        editor.insert("é", true);
        assert_eq!(editor.text, "éa🧬\n界");
    }

    #[test]
    fn long_dna_lines_and_wide_characters_clip_correctly() {
        let editor = Editor::new(format!("{}G", "A".repeat(70_000)));
        assert_eq!(editor.viewport(false, 0, 70_000, 5, 1), "G");
        let editor = Editor::new("界AT");
        assert_eq!(editor.viewport(false, 0, 1, 3, 1), " AT");
        assert_eq!(editor.viewport(true, 0, 0, 3, 1), "•••");
    }
}

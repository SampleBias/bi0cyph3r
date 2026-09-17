use std::{
    collections::VecDeque,
    path::Path,
    sync::mpsc::{self, Receiver},
    time::Instant,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{editor::Editor, theme::Theme};
use crate::{
    dna::EncodingMode,
    workbench::{self, Format, Operation, Output, Request, MODES},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Input,
    Mode,
    Password,
    K1,
    K2,
    Name,
    Structure,
    Output,
}

pub struct Workspace {
    pub input: Editor,
    pub password: Editor,
    pub k1: Editor,
    pub k2: Editor,
    pub name: Editor,
    pub mode: EncodingMode,
    pub structure: crate::plasmid::Structure,
    pub output: Option<Output>,
    pub scroll: u16,
    pub keys_saved: bool,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            input: Editor::default(),
            password: Editor::default(),
            k1: Editor::default(),
            k2: Editor::default(),
            name: Editor::new("biocypher_payload"),
            mode: EncodingMode::Basic,
            structure: crate::plasmid::Structure::Payload,
            output: None,
            scroll: 0,
            keys_saved: false,
        }
    }
}

pub enum Dialog {
    Help,
    Import(Editor),
    Export {
        path: Editor,
        format: Format,
        keys: bool,
    },
    Keys,
    ConfirmQuit,
    ConfirmRun,
}

type WorkerResult = Result<Output, String>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Home,
    Workbench,
    Guide,
    Themes,
}

pub struct Job {
    receiver: Receiver<WorkerResult>,
    pub page: usize,
    pub started: Instant,
}

pub struct App {
    pub screen: Screen,
    pub theme: Theme,
    pub menu_index: usize,
    pub theme_index: usize,
    pub guide_scroll: u16,
    pub rotation: f64,
    pub animate: bool,
    previous_screen: Screen,
    previous_theme: Theme,
    animation_updated: Instant,
    pub pages: [Workspace; 4],
    pub page: usize,
    pub focus: Field,
    pub editing: bool,
    pub dialog: Option<Dialog>,
    pub status: String,
    pub error: bool,
    pub activity: VecDeque<String>,
    pub completed: usize,
    pub job: Option<Job>,
    pub quit: bool,
    pub tick: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Home,
            theme: Theme::default(),
            menu_index: 0,
            theme_index: 0,
            guide_scroll: 0,
            rotation: 0.0,
            animate: true,
            previous_screen: Screen::Home,
            previous_theme: Theme::default(),
            animation_updated: Instant::now(),
            pages: std::array::from_fn(|_| Workspace::default()),
            page: 0,
            focus: Field::Input,
            editing: false,
            dialog: None,
            status: "Ready. Press i to write a message, or Ctrl+O to import a file.".into(),
            error: false,
            activity: VecDeque::new(),
            completed: 0,
            job: None,
            quit: false,
            tick: 0,
        }
    }
}

impl App {
    pub fn enter_workbench(&mut self) {
        self.screen = Screen::Workbench;
        self.editing = false;
    }

    fn open_guide(&mut self) {
        self.previous_screen = self.screen;
        self.screen = Screen::Guide;
        self.guide_scroll = 0;
    }

    fn open_themes(&mut self) {
        self.previous_screen = self.screen;
        self.previous_theme = self.theme;
        self.theme_index = Theme::ALL
            .iter()
            .position(|t| *t == self.theme)
            .unwrap_or(0);
        self.screen = Screen::Themes;
    }

    fn menu_action(&mut self) {
        match self.menu_index {
            0 => self.enter_workbench(),
            1 => self.open_guide(),
            _ => self.open_themes(),
        }
    }

    fn screen_key(&mut self, key: KeyEvent) {
        match self.screen {
            Screen::Home => match key.code {
                KeyCode::Char('s') => self.enter_workbench(),
                KeyCode::Char('g' | '?') => self.open_guide(),
                KeyCode::Char('t') => self.open_themes(),
                KeyCode::Enter => self.menu_action(),
                KeyCode::Right | KeyCode::Down | KeyCode::Tab | KeyCode::Char('j' | 'l') => {
                    self.menu_index = (self.menu_index + 1) % 3
                }
                KeyCode::Left | KeyCode::Up | KeyCode::BackTab | KeyCode::Char('k' | 'h') => {
                    self.menu_index = (self.menu_index + 2) % 3
                }
                KeyCode::Char(' ') => self.animate = !self.animate,
                KeyCode::Char('q') => self.request_quit(),
                _ => {}
            },
            Screen::Guide => match key.code {
                KeyCode::Esc | KeyCode::Char('g' | 'q') => self.screen = self.previous_screen,
                KeyCode::Char('s') => self.enter_workbench(),
                KeyCode::Down | KeyCode::Char('j') => {
                    self.guide_scroll = self.guide_scroll.saturating_add(1)
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.guide_scroll = self.guide_scroll.saturating_sub(1)
                }
                KeyCode::PageDown | KeyCode::Char(' ') => {
                    self.guide_scroll = self.guide_scroll.saturating_add(10)
                }
                KeyCode::PageUp => self.guide_scroll = self.guide_scroll.saturating_sub(10),
                KeyCode::Home => self.guide_scroll = 0,
                KeyCode::End => self.guide_scroll = u16::MAX,
                _ => {}
            },
            Screen::Themes => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        self.theme = self.previous_theme;
                        self.screen = self.previous_screen;
                        return;
                    }
                    KeyCode::Enter => {
                        self.screen = self.previous_screen;
                        self.message(
                            format!("{} theme applied to this session.", self.theme.name()),
                            false,
                        );
                        return;
                    }
                    KeyCode::Down | KeyCode::Right | KeyCode::Tab | KeyCode::Char('j') => {
                        self.theme_index = (self.theme_index + 1) % Theme::ALL.len()
                    }
                    KeyCode::Up | KeyCode::Left | KeyCode::BackTab | KeyCode::Char('k') => {
                        self.theme_index =
                            (self.theme_index + Theme::ALL.len() - 1) % Theme::ALL.len()
                    }
                    KeyCode::Char('1'..='5') => {
                        if let KeyCode::Char(c) = key.code {
                            self.theme_index = c as usize - '1' as usize;
                        }
                    }
                    KeyCode::Char(' ') => self.animate = !self.animate,
                    _ => {}
                }
                self.theme = Theme::ALL[self.theme_index];
            }
            Screen::Workbench => {}
        }
    }

    pub fn workspace(&self) -> &Workspace {
        &self.pages[self.page]
    }
    pub fn operation(&self) -> Operation {
        Operation::ALL[self.page]
    }

    pub fn fields(&self) -> Vec<Field> {
        let mut fields = vec![Field::Input];
        if self.operation() != Operation::Safety {
            fields.push(Field::Mode);
            if self.workspace().mode == EncodingMode::Secure {
                fields.push(Field::Password);
            }
            if self.workspace().mode == EncodingMode::SplitKey
                && self.operation() == Operation::Decode
            {
                fields.extend([Field::K1, Field::K2]);
            }
        }
        if self.operation() == Operation::Plasmid {
            fields.push(Field::Name);
            fields.push(Field::Structure);
        }
        fields.push(Field::Output);
        fields
    }

    pub fn editor(&mut self) -> Option<&mut Editor> {
        let workspace = &mut self.pages[self.page];
        match self.focus {
            Field::Input => Some(&mut workspace.input),
            Field::Password => Some(&mut workspace.password),
            Field::K1 => Some(&mut workspace.k1),
            Field::K2 => Some(&mut workspace.k2),
            Field::Name => Some(&mut workspace.name),
            _ => None,
        }
    }

    pub fn message(&mut self, text: impl Into<String>, error: bool) {
        self.status = text.into();
        self.error = error;
    }

    fn select_page(&mut self, page: usize) {
        self.page = page;
        self.focus = Field::Input;
        self.editing = false;
    }

    fn cycle_focus(&mut self, backwards: bool) {
        let fields = self.fields();
        let i = fields.iter().position(|f| *f == self.focus).unwrap_or(0);
        self.focus = fields[(i + if backwards { fields.len() - 1 } else { 1 }) % fields.len()];
        self.editing = false;
    }

    fn cycle_mode(&mut self, backwards: bool) {
        if self.operation() == Operation::Safety {
            return;
        }
        let workspace = &mut self.pages[self.page];
        let i = MODES.iter().position(|m| *m == workspace.mode).unwrap_or(0);
        workspace.mode = MODES[(i + if backwards { 3 } else { 1 }) % 4];
        if !self.fields().contains(&self.focus) {
            self.focus = Field::Input;
        }
        self.message(match self.workspace().mode {
            EncodingMode::Basic => "Basic: maps UTF-8 text to DNA; no encryption.",
            EncodingMode::Nanopore => "Nanopore: triplets with parity, redundancy and padding.",
            EncodingMode::Secure => "Secure: AES-256-CBC. Tab to the password field, then Enter to edit.",
            EncodingMode::SplitKey => "Split key: both K1 and K2 are required to decode. Ctrl+K exports generated keys.",
        }, false);
    }

    fn unsaved_keys(&self) -> bool {
        self.pages
            .iter()
            .any(|p| !p.keys_saved && p.output.as_ref().is_some_and(|o| o.keys.is_some()))
    }

    fn request_quit(&mut self) {
        if self.unsaved_keys() || self.job.is_some() {
            self.dialog = Some(Dialog::ConfirmQuit);
        } else {
            self.quit = true;
        }
    }

    pub fn start(&mut self, confirmed: bool) {
        if self.job.is_some() {
            return;
        }
        let workspace = self.workspace();
        if !confirmed
            && !workspace.keys_saved
            && workspace.output.as_ref().is_some_and(|o| o.keys.is_some())
        {
            self.dialog = Some(Dialog::ConfirmRun);
            return;
        }
        let request = Request {
            operation: self.operation(),
            input: workspace.input.text.clone(),
            mode: workspace.mode,
            password: workspace.password.text.clone(),
            k1: workspace.k1.text.clone(),
            k2: workspace.k2.text.clone(),
            name: workspace.name.text.clone(),
            structure: workspace.structure,
        };
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let result = workbench::execute(&request).map_err(|e| format!("{e:#}"));
            let _ = sender.send(result);
        });
        self.job = Some(Job {
            receiver,
            page: self.page,
            started: Instant::now(),
        });
        self.editing = false;
        self.message(format!("{} in progress…", self.operation().title()), false);
    }

    pub fn poll(&mut self) {
        let now = Instant::now();
        if self.animate {
            self.rotation = (self.rotation
                + now.duration_since(self.animation_updated).as_secs_f64() * 0.9)
                % std::f64::consts::TAU;
        }
        self.animation_updated = now;
        self.tick = self.tick.wrapping_add(1);
        let Some(job) = &self.job else {
            return;
        };
        let result = match job.receiver.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                Err("Operation stopped unexpectedly; your input is intact.".into())
            }
        };
        let page = job.page;
        let elapsed = job.started.elapsed();
        self.job = None;
        match result {
            Ok(output) => {
                let detail = format!("{} · {} bp", output.operation.title(), output.stats.length);
                let has_keys = output.keys.is_some();
                self.pages[page].output = Some(output);
                self.pages[page].scroll = 0;
                self.pages[page].keys_saved = false;
                self.completed += 1;
                self.activity.push_front(detail.clone());
                self.activity.truncate(6);
                self.message(
                    if has_keys {
                        format!("{detail}. Ctrl+K exports your keys; keep K1 and K2 separately.")
                    } else {
                        format!(
                            "{detail} · {:.2}s · Ctrl+S to export",
                            elapsed.as_secs_f64()
                        )
                    },
                    false,
                );
            }
            Err(error) => self.message(error, true),
        }
    }

    fn open_export(&mut self, keys: bool) {
        let Some(output) = &self.workspace().output else {
            self.message("Run an operation before exporting.", true);
            return;
        };
        if keys && output.keys.is_none() {
            self.message("This result has no split keys.", true);
            return;
        }
        let format = if self.operation() == Operation::Plasmid {
            Format::Fasta
        } else if self.operation() == Operation::Safety {
            Format::Json
        } else {
            Format::Txt
        };
        self.dialog = Some(Dialog::Export {
            path: Editor::new(if keys {
                "biocypher.keys.json".to_string()
            } else {
                format!("biocypher.{}", format.extension())
            }),
            format,
            keys,
        });
        self.editing = false;
    }

    fn transfer(&mut self, target: usize) {
        let Some(output) = self.workspace().output.clone() else {
            self.message("Run an operation before sending its result.", true);
            return;
        };
        let password = self.workspace().password.clone();
        let workspace = &mut self.pages[target];
        workspace.input = Editor::new(output.sequence);
        workspace.input.cursor = 0;
        workspace.mode = output.mode;
        workspace.password = password;
        if let Some(keys) = output.keys {
            workspace.k1 = Editor::new(keys.k1_base64);
            workspace.k2 = Editor::new(keys.k2_base64);
        }
        self.select_page(target);
        self.message("Sequence loaded. Ctrl+R to run.", false);
    }

    pub fn paste(&mut self, text: &str) {
        match &mut self.dialog {
            Some(Dialog::Import(editor)) | Some(Dialog::Export { path: editor, .. }) => {
                editor.insert(text, false)
            }
            Some(_) => {}
            None if self.screen == Screen::Workbench && self.job.is_none() && self.editing => {
                let multiline = self.focus == Field::Input;
                if let Some(editor) = self.editor() {
                    editor.insert(text, multiline);
                }
            }
            None => {}
        }
    }

    pub fn key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl && key.code == KeyCode::Char('c') {
            if matches!(self.dialog, Some(Dialog::ConfirmQuit)) {
                self.quit = true;
            } else {
                self.request_quit();
            }
            return;
        }
        if self.dialog.is_some() {
            self.dialog_key(key);
            return;
        }
        if key.code == KeyCode::F(10) || (ctrl && key.code == KeyCode::Char('q')) {
            self.request_quit();
            return;
        }
        if key.code == KeyCode::F(9)
            && self.screen != Screen::Themes
            && self.screen != Screen::Guide
        {
            self.open_themes();
            return;
        }
        if self.screen != Screen::Workbench {
            self.screen_key(key);
            return;
        }
        if key.code == KeyCode::F(1) {
            self.select_page(0);
            return;
        }
        if key.code == KeyCode::F(2) {
            self.select_page(1);
            return;
        }
        if key.code == KeyCode::F(3) {
            self.select_page(2);
            return;
        }
        if key.code == KeyCode::F(4) {
            self.select_page(3);
            return;
        }
        if self.job.is_some() {
            if key.code == KeyCode::Char('?') {
                self.dialog = Some(Dialog::Help);
            }
            return;
        }
        if ctrl {
            match key.code {
                KeyCode::Char('r') => {
                    self.start(false);
                    return;
                }
                KeyCode::Char('o') => {
                    self.dialog = Some(Dialog::Import(Editor::default()));
                    return;
                }
                KeyCode::Char('s') => {
                    self.open_export(false);
                    return;
                }
                KeyCode::Char('k') => {
                    self.open_export(true);
                    return;
                }
                _ => {}
            }
        }
        match key.code {
            KeyCode::F(5) => {
                self.start(false);
                return;
            }
            KeyCode::F(6) => {
                self.open_export(false);
                return;
            }
            KeyCode::Tab => {
                self.cycle_focus(false);
                return;
            }
            KeyCode::BackTab => {
                self.cycle_focus(true);
                return;
            }
            KeyCode::Esc => {
                if self.editing {
                    self.editing = false;
                } else {
                    self.screen = Screen::Home;
                }
                return;
            }
            _ => {}
        }
        if self.editing {
            let multiline = self.focus == Field::Input;
            if key.code == KeyCode::Enter && !multiline {
                self.editing = false;
            } else if let Some(editor) = self.editor() {
                editor.key(key, multiline);
            }
            return;
        }
        match key.code {
            KeyCode::Char('1'..='4') => {
                if let KeyCode::Char(c) = key.code {
                    self.select_page(c as usize - '1' as usize);
                }
            }
            KeyCode::Char('q') => self.request_quit(),
            KeyCode::Char('?') => self.dialog = Some(Dialog::Help),
            KeyCode::Char('g') => self.open_guide(),
            KeyCode::Char('t') => self.open_themes(),
            KeyCode::Char('m') => self.cycle_mode(false),
            KeyCode::Char('x') if self.operation() == Operation::Plasmid => {
                self.pages[self.page].structure = self.workspace().structure.next();
            }
            KeyCode::Left if self.focus == Field::Mode => self.cycle_mode(true),
            KeyCode::Right if self.focus == Field::Mode => self.cycle_mode(false),
            KeyCode::Char('i' | 'e') | KeyCode::Enter => {
                if self.focus == Field::Mode {
                    self.cycle_mode(false);
                } else if self.focus == Field::Structure {
                    self.pages[self.page].structure = self.workspace().structure.next();
                } else if self.editor().is_some() {
                    self.editing = true;
                }
            }
            KeyCode::Char('d') => self.transfer(1),
            KeyCode::Char('s') => self.transfer(2),
            KeyCode::Char('v') => {
                if self
                    .workspace()
                    .output
                    .as_ref()
                    .is_some_and(|o| o.keys.is_some())
                {
                    self.dialog = Some(Dialog::Keys);
                }
            }
            KeyCode::PageDown => self.scroll(10),
            KeyCode::PageUp => self.scroll(-10),
            KeyCode::Down | KeyCode::Char('j') if self.focus == Field::Output => self.scroll(1),
            KeyCode::Up | KeyCode::Char('k') if self.focus == Field::Output => self.scroll(-1),
            _ => {}
        }
    }

    fn scroll(&mut self, delta: i16) {
        let workspace = &mut self.pages[self.page];
        workspace.scroll = workspace.scroll.saturating_add_signed(delta);
    }

    fn dialog_key(&mut self, key: KeyEvent) {
        let Some(mut dialog) = self.dialog.take() else {
            return;
        };
        if key.code == KeyCode::Esc {
            return;
        }
        match &mut dialog {
            Dialog::Help | Dialog::Keys => {
                return;
            }
            Dialog::ConfirmQuit => {
                if key.code == KeyCode::Char('y') {
                    self.quit = true;
                } else if key.code != KeyCode::Char('n') {
                    self.dialog = Some(dialog);
                }
                return;
            }
            Dialog::ConfirmRun => {
                if key.code == KeyCode::Char('y') {
                    self.start(true);
                } else if key.code != KeyCode::Char('n') {
                    self.dialog = Some(dialog);
                }
                return;
            }
            Dialog::Import(editor) => {
                if key.code == KeyCode::Enter {
                    match workbench::read_input(Path::new(&editor.text)) {
                        Ok(text) => {
                            self.pages[self.page].input = Editor::new(text);
                            self.pages[self.page].input.cursor = 0;
                            self.focus = Field::Input;
                            self.editing = false;
                            self.message("File imported. Ctrl+R to run.", false);
                            return;
                        }
                        Err(error) => self.message(format!("{error:#}"), true),
                    }
                } else {
                    editor.key(key, false);
                }
            }
            Dialog::Export { path, format, keys } => {
                if key.code == KeyCode::Tab && !*keys {
                    let old_extension = format.extension();
                    *format = format.next();
                    let suffix = format!(".{old_extension}");
                    if let Some(stem) = path.text.strip_suffix(&suffix) {
                        *path = Editor::new(format!("{stem}.{}", format.extension()));
                    }
                } else if key.code == KeyCode::Enter {
                    if let Some(output) = &self.workspace().output {
                        let contents = if *keys {
                            serde_json::to_string_pretty(&output.keys).map_err(anyhow::Error::from)
                        } else {
                            output.export(*format)
                        };
                        let result =
                            contents.and_then(|s| workbench::save_new(Path::new(&path.text), &s));
                        match result {
                            Ok(()) => {
                                if *keys {
                                    self.pages[self.page].keys_saved = true;
                                }
                                self.message(
                                    format!(
                                        "Saved {}{}",
                                        path.text,
                                        if *keys {
                                            " · Store K1 and K2 separately."
                                        } else {
                                            ""
                                        }
                                    ),
                                    false,
                                );
                                return;
                            }
                            Err(error) => self.message(format!("{error:#}"), true),
                        }
                    }
                } else {
                    path.key(key, false);
                }
            }
        }
        self.dialog = Some(dialog);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn typing_does_not_trigger_navigation_and_tabs_retain_inputs() {
        let mut app = App::default();
        app.enter_workbench();
        app.key(key(KeyCode::Char('i')));
        for c in "q1234msd?".chars() {
            app.key(key(KeyCode::Char(c)));
        }
        assert!(!app.quit);
        assert_eq!(app.page, 0);
        assert_eq!(app.workspace().input.text, "q1234msd?");
        app.key(key(KeyCode::Esc));
        app.key(key(KeyCode::Char('2')));
        app.key(key(KeyCode::Char('1')));
        assert_eq!(app.workspace().input.text, "q1234msd?");
    }

    #[test]
    fn passwords_and_paste_are_not_commands() {
        let mut app = App::default();
        app.enter_workbench();
        app.pages[0].mode = EncodingMode::Secure;
        app.focus = Field::Password;
        app.editing = true;
        app.paste("qsecret\npassword");
        assert_eq!(app.workspace().password.text, "qsecretpassword");
        assert!(!app.quit);
    }

    #[test]
    fn worker_result_returns_to_original_workspace_and_unsaved_keys_are_guarded() {
        let mut app = App::default();
        app.enter_workbench();
        app.pages[0].input = Editor::new("Session roundtrip");
        app.pages[0].mode = EncodingMode::SplitKey;
        app.key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        app.key(key(KeyCode::F(2)));
        let deadline = Instant::now() + std::time::Duration::from_secs(5);
        while app.job.is_some() && Instant::now() < deadline {
            app.poll();
            std::thread::yield_now();
        }
        assert!(app.job.is_none());
        assert!(app.pages[0].output.as_ref().unwrap().keys.is_some());
        assert!(app.pages[1].output.is_none());
        app.key(key(KeyCode::Char('q')));
        assert!(matches!(app.dialog, Some(Dialog::ConfirmQuit)));
        assert!(!app.quit);
        app.key(key(KeyCode::Esc));
        app.key(key(KeyCode::F(1)));
        app.key(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::CONTROL));
        assert!(matches!(app.dialog, Some(Dialog::ConfirmRun)));
        assert!(app.job.is_none());
        app.key(key(KeyCode::Esc));
        app.key(key(KeyCode::Char('d')));
        assert!(!app.workspace().k1.text.is_empty());
        assert!(!app.workspace().k2.text.is_empty());
    }

    #[test]
    fn startup_menu_guide_and_home_preserve_the_workbench() {
        let mut app = App::default();
        assert_eq!(app.screen, Screen::Home);
        app.key(key(KeyCode::Char('g')));
        assert_eq!(app.screen, Screen::Guide);
        app.key(key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Home);
        app.key(key(KeyCode::Char('s')));
        assert_eq!(app.screen, Screen::Workbench);
        assert_eq!(app.operation(), Operation::Encode);
        app.key(key(KeyCode::Char('i')));
        app.paste("saved session");
        app.key(key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Workbench);
        app.key(key(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Home);
        app.key(key(KeyCode::Enter));
        assert_eq!(app.workspace().input.text, "saved session");
    }

    #[test]
    fn theme_preview_can_be_cancelled_or_applied_without_changing_inputs() {
        let mut app = App::default();
        app.key(key(KeyCode::Char('t')));
        app.key(key(KeyCode::Char('3')));
        assert_eq!(app.theme, Theme::Paper);
        app.key(key(KeyCode::Esc));
        assert_eq!(app.theme, Theme::Original);
        assert_eq!(app.screen, Screen::Home);
        app.key(key(KeyCode::Char('t')));
        app.key(key(KeyCode::Char('2')));
        app.key(key(KeyCode::Enter));
        assert_eq!(app.theme, Theme::Crimson);
        app.key(key(KeyCode::Char('s')));
        app.pages[0].input = Editor::new("keep me");
        app.key(key(KeyCode::F(9)));
        app.key(key(KeyCode::Char('4')));
        app.key(key(KeyCode::Enter));
        assert_eq!(app.screen, Screen::Workbench);
        assert_eq!(app.theme, Theme::Monochrome);
        assert_eq!(app.workspace().input.text, "keep me");
    }

    #[test]
    fn paused_animation_does_not_advance_with_input_events() {
        let mut app = App::default();
        app.key(key(KeyCode::Char(' ')));
        let phase = app.rotation;
        for _ in 0..10 {
            app.key(key(KeyCode::Right));
            app.poll();
        }
        assert_eq!(app.rotation, phase);
        assert!(!app.animate);
    }
}

mod macho;
mod open;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};
use std::{path::PathBuf, str::FromStr};

use crate::{
    macho::MachoWidget,
    open::{BinaryFile, open_binary_file},
};

fn main() -> Result<(), String> {
    let mut terminal = ratatui::init();
    Mule::new().run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}

#[derive(Debug)]
enum InputMode {
    Command,     // Focus in on the command line
    Interactive, // Focus is on the display widget
}

struct BinaryState {
    path: PathBuf,
    file: BinaryFile,
}

struct ProjectState {
    binary: Option<BinaryState>,
}

struct Mule {
    project_state: ProjectState,
    input: String,
    input_mode: InputMode,
    character_index: usize,
    exit: bool,
}

impl Mule {
    pub fn new() -> Mule {
        let project_state = ProjectState { binary: None };

        Mule {
            project_state,
            input: String::new(),
            input_mode: InputMode::Command,
            character_index: 0,
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), String> {
        while !self.exit {
            terminal
                .draw(|frame| self.draw(frame))
                .map_err(|e| e.to_string())?;
            if self.handle_events()? {
                return Ok(()); // quit
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn handle_events(&mut self) -> Result<bool, String> {
        if let Event::Key(key) = event::read().map_err(|e| e.to_string())? {
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }
            match self.input_mode {
                InputMode::Command => match key.code {
                    KeyCode::Enter => {
                        if self.exec_command()? {
                            return Ok(true);
                        }
                        self.input_mode = InputMode::Interactive;
                    }
                    KeyCode::Char(to_insert) => self.enter_char(to_insert),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    _ => { /* ignore */ }
                },
                InputMode::Interactive => {
                    match key.code {
                        KeyCode::Esc => self.input_mode = InputMode::Command,
                        _ => { /* ignore */ }
                    }
                    // TODO forward event to current widget
                }
            }
        }
        Ok(false)
    }

    fn exec_command(&mut self) -> Result<bool, String> {
        if self.input == ":q" {
            return Ok(true);
        }

        let input_cmd = self.input.clone();

        if input_cmd.starts_with(":o") {
            let mut iter = input_cmd.split_whitespace();
            iter.next();

            let file_path = iter.next().expect("file_path");
            let path = PathBuf::from_str(file_path).map_err(|e| e.to_string())?;
            let binary_file = open_binary_file(&path)?;
            self.project_state.binary = Some(BinaryState {
                path,
                file: binary_file,
            })
        }

        self.input.clear();
        self.character_index = 0;

        Ok(false)
    }
}

impl Widget for &Mule {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout =
            Layout::vertical([Constraint::Max(3), Constraint::Min(0), Constraint::Max(3)]);
        let [header, content, command] = main_layout.areas(area);

        let header_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title("Binary");

        let binary_str = if let Some(binary_state) = self.project_state.binary.as_ref() {
            let binary_str = binary_state.path.to_str().unwrap();
            // TODO show real info from loaded binary here
            &format!("{} (Mach-O, arm64, executable)", binary_str)
        } else {
            "<no binary loaded>"
        };

        Paragraph::new(binary_str.bold())
            .block(header_block)
            .render(header, buf);

        if let Some(binary_state) = self.project_state.binary.as_ref() {
            match &binary_state.file {
                BinaryFile::Macho(macho) => {
                    let mut widget = MachoWidget::new(macho);
                    widget.render(content, buf);
                }
            }
        } else {
            let placeholder_block = Block::bordered().border_type(BorderType::Plain);
            Paragraph::new(
                "No binary loaded, please load a binary for inspection with the :o command",
            )
            .fg(Color::LightRed)
            .block(placeholder_block)
            .render(content, buf)
        }

        let command_block = Block::bordered().border_type(BorderType::Plain);
        Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Interactive => Style::default(),
                InputMode::Command => Style::default().fg(Color::Yellow),
            })
            .block(command_block)
            .render(command, buf);
    }
}

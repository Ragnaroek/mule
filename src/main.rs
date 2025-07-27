use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};
use std::{io, path::PathBuf};

fn main() -> Result<(), String> {
    let mut terminal = ratatui::init();
    Mule::new().run(&mut terminal).map_err(|e| e.to_string())?;
    ratatui::restore();
    Ok(())
}

enum InputMode {
    Normal,
    Editing,
}

struct ProjectState {
    binary_path: Option<PathBuf>,
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
        let project_state = ProjectState { binary_path: None };

        Mule {
            project_state,
            input: String::new(),
            input_mode: InputMode::Normal,
            character_index: 0,
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
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

    fn handle_events(&mut self) -> io::Result<bool> {
        if let Event::Key(key) = event::read()? {
            match self.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char(':') => {
                        self.enter_char(':');
                        self.input_mode = InputMode::Editing;
                    }
                    _ => {}
                },
                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        if self.exec_command() {
                            return Ok(true);
                        }
                    }
                    KeyCode::Char(to_insert) => self.enter_char(to_insert),
                    KeyCode::Backspace => self.delete_char(),
                    KeyCode::Left => self.move_cursor_left(),
                    KeyCode::Right => self.move_cursor_right(),
                    KeyCode::Esc => self.input_mode = InputMode::Normal,
                    _ => {}
                },
                InputMode::Editing => {}
            }
        }
        Ok(false)
    }

    fn exec_command(&mut self) -> bool {
        if self.input == ":q" {
            return true;
        }

        let input_cmd = self.input.clone();

        // TODO parse input here

        self.input.clear();
        self.character_index = 0;

        false
    }
}

impl Widget for &Mule {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout =
            Layout::vertical([Constraint::Max(3), Constraint::Min(0), Constraint::Max(3)]);
        let [header, content, command] = main_layout.areas(area);

        let content_layout =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
        let [content_file, content_detail] = content_layout.areas(content);

        // TODO Hights have to be computed dynamically from the Mach-O file
        let file_layout =
            Layout::vertical([Constraint::Max(3), Constraint::Max(10), Constraint::Max(30)]);
        let [mach_header, mach_commands, mach_segments] = file_layout.areas(content_file);

        let header_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title("Binary");

        let binary_str = if self.project_state.binary_path.is_none() {
            "<no binary loaded>"
        } else {
            // TODO show real info from loaded binary here
            "iw (Mach-O, arm64, executable)"
        };

        Paragraph::new(binary_str.bold())
            .block(header_block)
            .render(header, buf);

        if self.project_state.binary_path.is_none() {
            let placeholder_block = Block::bordered().border_type(BorderType::Plain);
            Paragraph::new(
                "No binary loaded, please load a binary for inspection with the :o command",
            )
            .fg(Color::LightRed)
            .block(placeholder_block)
            .render(content, buf)
        } else {
            // TODO put this into the macho.rs file
            Block::bordered()
                .border_type(BorderType::Plain)
                //.style(Style::new().black().on_white())
                .title("Header")
                .render(mach_header, buf);

            Block::bordered()
                .border_type(BorderType::Plain)
                .style(Style::new().black().on_white())
                .title("Load Commands")
                .render(mach_commands, buf);

            Block::bordered()
                .border_type(BorderType::Plain)
                //.style(Style::new().black().on_white())
                .title("Segments")
                .render(mach_segments, buf);

            Block::bordered()
                .border_type(BorderType::Plain)
                .title("Details")
                .render(content_detail, buf);
        }

        let command_block = Block::bordered().border_type(BorderType::Plain);
        Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(command_block)
            .render(command, buf);
    }
}

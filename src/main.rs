use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Widget},
};
use std::io;

fn main() -> Result<(), String> {
    let mut terminal = ratatui::init();
    Mule::new().run(&mut terminal).map_err(|e| e.to_string())?;
    ratatui::restore();
    Ok(())
}

struct Mule {
    counter: u8,
    exit: bool,
}

impl Mule {
    pub fn new() -> Mule {
        Mule {
            counter: 0,
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Left => self.counter += 1,
            KeyCode::Right => self.counter -= 1,
            _ => {}
        }
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
        Paragraph::new("iw (Mach-O, arm64, executable".bold())
            .block(header_block)
            .render(header, buf);

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

        let command_block = Block::bordered().border_type(BorderType::Plain);
        Paragraph::new(":dwarf")
            .block(command_block)
            .render(command, buf);
    }
}

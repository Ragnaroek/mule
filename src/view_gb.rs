use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, List, ListState, Paragraph, StatefulWidget, Widget},
};

use mule_gb::GBBinary;

pub struct GBInteractiveState {
    pub command_state: ListState,
}

impl GBInteractiveState {
    pub fn new() -> GBInteractiveState {
        let mut command_state = ListState::default();
        command_state.select(Some(0));
        GBInteractiveState { command_state }
    }

    pub fn handle_key(&mut self, key: KeyCode) {}
}

pub struct GBWidget<'a> {
    pub gb_binary: &'a GBBinary,
    pub state: &'a mut GBInteractiveState,
}

impl<'a> GBWidget<'a> {
    pub fn new(gb_binary: &'a GBBinary, state: &'a mut GBInteractiveState) -> GBWidget<'a> {
        GBWidget { gb_binary, state }
    }
}

impl<'a> Widget for &mut GBWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content_layout =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]);
        let [content_file, content_detail] = content_layout.areas(area);

        let file_layout = Layout::vertical([Constraint::Max(3), Constraint::Max(3)]);
        let [gb_vectors, gb_header] = file_layout.areas(content_file);

        let header_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title("Header");

        Paragraph::new(format!("TODO GB Header",))
            .block(header_block)
            .render(gb_header, buf);
    }
}

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, List, ListState, Paragraph, StatefulWidget, Widget},
};

use mule_gb::{GBBinary, ROMSize, num_banks};

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

        let file_layout =
            Layout::vertical([Constraint::Max(3), Constraint::Max(3), Constraint::Fill(1)]);
        let [gb_vectors, gb_header, gb_banks] = file_layout.areas(content_file);

        let vector_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title("Interrupt Vectors");

        Paragraph::new(format!("TODO Vectors",))
            .block(vector_block)
            .render(gb_vectors, buf);

        let header_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title("Header");

        Paragraph::new(format!(
            "title:{} | type:{:?}",
            self.gb_binary.header.game_title, self.gb_binary.header.cartridge_type
        ))
        .block(header_block)
        .render(gb_header, buf);

        let bank_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title(format!(
                "Banks ({})",
                num_banks(self.gb_binary.header.rom_size)
            ));

        let cmd_list = List::new(bank_list(self.gb_binary))
            .block(bank_block)
            .highlight_style(Style::new().black().on_white());
        StatefulWidget::render(cmd_list, gb_banks, buf, &mut ListState::default());

        let detail_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title("Details")
            .render(content_detail, buf);
    }
}

fn bank_list(binary: &GBBinary) -> Vec<String> {
    let n = num_banks(binary.header.rom_size);
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        result.push(format!("Bank {}", i));
    }
    result
}

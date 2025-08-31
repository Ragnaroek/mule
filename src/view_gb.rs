use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, BorderType, List, ListState, Paragraph, StatefulWidget, Widget, WidgetRef},
};

use mule_gb::{GBBinary, num_banks};

use crate::{
    InteractiveCommand,
    hex::Hex,
    view::{style_focus, style_normal},
};

#[derive(PartialEq, Clone, Copy)]
enum Focus {
    None,
    Vectors,
    Header,
    Banks,
}

static FOCUS_CYCLE_ORDER: [Focus; 3] = [Focus::Vectors, Focus::Header, Focus::Banks];

pub struct GBInteractiveState {
    previous_focus: Focus,
    focus_on: Focus,
    bank_list_state: ListState,
}

impl GBInteractiveState {
    pub fn new() -> GBInteractiveState {
        let mut bank_list_state = ListState::default();
        bank_list_state.select(Some(0));
        GBInteractiveState {
            bank_list_state,
            previous_focus: Focus::None,
            focus_on: Focus::Banks,
        }
    }

    pub fn handle_command(&mut self, command: InteractiveCommand) {
        match command {
            InteractiveCommand::Key(key) => {
                match key {
                    KeyCode::Tab => self.move_focus(1),
                    KeyCode::BackTab => self.move_focus(-1),
                    KeyCode::Down => {
                        if self.focus_on == Focus::Banks {
                            self.bank_list_state.select_next();
                        }
                    }
                    KeyCode::Up => {
                        if self.focus_on == Focus::Banks {
                            self.bank_list_state.select_previous();
                        }
                    }
                    _ => { /* ignore */ }
                }
            }
            InteractiveCommand::Focus => {
                self.focus_on = self.previous_focus;
            }
            InteractiveCommand::Unfocus => {
                self.previous_focus = self.focus_on;
                self.focus_on = Focus::None;
            }
        }
    }

    fn move_focus(&mut self, dir: isize) {
        let mut ix_focus = 0;
        for i in 0..FOCUS_CYCLE_ORDER.len() {
            if FOCUS_CYCLE_ORDER[i] == self.focus_on {
                ix_focus = i as isize;
            }
        }
        ix_focus += dir;
        let ix = if ix_focus < 0 {
            (FOCUS_CYCLE_ORDER.len() as isize + ix_focus) as usize
        } else {
            ix_focus as usize % FOCUS_CYCLE_ORDER.len()
        };

        self.focus_on = FOCUS_CYCLE_ORDER[ix];
    }
}

pub struct GBWidget<'a> {
    pub gb_binary: &'a GBBinary,
    pub state: &'a mut GBInteractiveState,
}

impl<'a> GBWidget<'a> {
    pub fn new(gb_binary: &'a GBBinary, state: &'a mut GBInteractiveState) -> GBWidget<'a> {
        GBWidget { gb_binary, state }
    }

    fn focus_style(&self, focus: Focus) -> Style {
        if self.state.focus_on == focus {
            style_focus()
        } else {
            style_normal()
        }
    }

    fn render_detail_view(&self, content_detail: Rect, buf: &mut Buffer) {
        let detail_block = Block::bordered()
            .border_type(BorderType::Plain)
            .title("Details");

        match self.state.focus_on {
            Focus::None => {}
            Focus::Vectors => {} // TODO
            Focus::Header => {}  // TODO
            Focus::Banks => {
                let selected = self.state.bank_list_state.selected();
                if let Some(selected_pos) = selected {
                    let bank = &self.gb_binary.bank_data[selected_pos];
                    let hex = &Hex::new(bank).block(detail_block);
                    hex.render_ref(content_detail, buf);
                }
            }
        }
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
            .style(self.focus_style(Focus::Vectors))
            .title("Interrupt Vectors");

        Paragraph::new(format!("TODO Vectors",))
            .block(vector_block)
            .render(gb_vectors, buf);

        let header_block = Block::bordered()
            .border_type(BorderType::Plain)
            .style(self.focus_style(Focus::Header))
            .title("Header");

        Paragraph::new(format!(
            "title:{} | type:{:?}",
            self.gb_binary.header.game_title, self.gb_binary.header.cartridge_type
        ))
        .block(header_block)
        .render(gb_header, buf);

        let bank_block = Block::bordered()
            .border_type(BorderType::Plain)
            .style(self.focus_style(Focus::Banks))
            .title(format!(
                "Banks ({})",
                num_banks(self.gb_binary.header.rom_size)
            ));

        let cmd_list = List::new(bank_list(self.gb_binary))
            .block(bank_block)
            .highlight_style(Style::new().black().on_white());
        StatefulWidget::render(cmd_list, gb_banks, buf, &mut self.state.bank_list_state);

        self.render_detail_view(content_detail, buf);
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

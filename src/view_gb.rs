use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    widgets::{
        Block, BorderType, List, ListState, Paragraph, Row, StatefulWidget, Table, Widget,
        WidgetRef,
    },
};

use mule_gb::{DestinationCode, GBBinary, GBCFlag, Header, RAMSize, ROMSize, SGBFlag, num_banks};
use psy::dasm::gb;

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

/// Cached disassembles that are only computed once
struct GBDisassembles {
    entry_point: Vec<String>,
}

pub struct GBInteractiveState {
    previous_focus: Focus,
    focus_on: Focus,
    bank_list_state: ListState,
    disassembles: GBDisassembles,
}

impl GBInteractiveState {
    pub fn new(binary: &GBBinary) -> GBInteractiveState {
        let mut bank_list_state = ListState::default();
        bank_list_state.select(Some(0));

        let entry_point = match gb::disassemble(&binary.header.entry_point) {
            Err(err) => vec![format!("Err disassemble: {}", err)],
            Ok(dis) => dis,
        };

        GBInteractiveState {
            bank_list_state,
            previous_focus: Focus::None,
            focus_on: Focus::Header,
            disassembles: GBDisassembles { entry_point },
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
            Focus::None => { /* do nothing */ }
            Focus::Vectors => {} // TODO
            Focus::Header => self.render_header_detail(detail_block, content_detail, buf),
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

    fn render_header_detail(&self, block: Block, content_detail: Rect, buf: &mut Buffer) {
        let entry_text = self.state.disassembles.entry_point.join("");
        let manufacturer_text = manufacturer_display(&self.gb_binary.header.manufacturer_code);
        let licensee_text = &format!("{:?}", self.gb_binary.header.licensee_code);
        let cartridge_text = &format!("{:?}", self.gb_binary.header.cartridge_type);
        let rom_text = rom_display(self.gb_binary.header.rom_size);
        let ram_text = ram_display(self.gb_binary.header.ram_size);
        let dest_text = dest_code_display(self.gb_binary.header.destination_code);
        let rom_version_text = &format!("{}", self.gb_binary.header.rom_version);
        let checksum_text = &format!("{}", self.gb_binary.header.checksum);
        let global_checksum_text = &format!("{}", self.gb_binary.header.global_checksum);
        let rows = [
            Row::new(vec!["Entry Point:", &entry_text]),
            Row::new(vec!["Game Title:", &self.gb_binary.header.game_title]),
            Row::new(vec!["Manufacturer Code:", manufacturer_text]),
            Row::new(vec![
                "GBC Flag:",
                gbc_flag_display(self.gb_binary.header.gbc_flag),
            ]),
            Row::new(vec!["Licensee Code:", licensee_text]),
            Row::new(vec![
                "Super Gameboy Flag:",
                sgb_flag_display(self.gb_binary.header.sgb_flag),
            ]),
            Row::new(vec!["Cartridge Type:", cartridge_text]),
            Row::new(vec!["ROM Size: ", rom_text]),
            Row::new(vec!["RAM Size:", ram_text]),
            Row::new(vec!["Destination Code:", dest_text]),
            Row::new(vec!["ROM Version:", rom_version_text]),
            Row::new(vec!["Checksum:", checksum_text]),
            Row::new(vec!["Global Checksum:", global_checksum_text]),
        ];
        let widths = [Constraint::Length(22), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(block);
        Widget::render(table, content_detail, buf);
    }
}

fn dest_code_display(dest_code: DestinationCode) -> &'static str {
    match dest_code {
        DestinationCode::Japanese => "Japanese",
        DestinationCode::NonJapanese => "No Japanese",
    }
}

fn ram_display(ram: RAMSize) -> &'static str {
    match ram {
        RAMSize::None => "No RAM",
        RAMSize::KB2 => "2 KiB",
        RAMSize::KB8 => "8 KiB",
        RAMSize::KB32 => "32 KiB",
        RAMSize::KB64 => "64 KiB",
        RAMSize::KB128 => "128 KiB",
    }
}

fn rom_display(rom: ROMSize) -> &'static str {
    match rom {
        ROMSize::NoBanking => "No Banking (32KiB)",
        ROMSize::Banks4 => "4 Banks (64 KiB)",
        ROMSize::Banks8 => "8 Banks (128 KiB)",
        ROMSize::Banks16 => "16 Banks (256 KiB)",
        ROMSize::Banks32 => "32 Banks (512 KiB)",
        ROMSize::Banks64 => "64 Banks (1 MiB)",
        ROMSize::Banks72 => "72 Banks (1.1 MiB)",
        ROMSize::Banks80 => "80 Banks (1.2 MiB)",
        ROMSize::Banks96 => "96 Banks (1.5 MiB)",
        ROMSize::Banks128 => "128 Banks (2 MiB)",
        ROMSize::Banks256 => "256 Banks (4 MiB)",
        ROMSize::Banks512 => "512 Banks (8 MiB)",
    }
}

fn manufacturer_display<'a>(code: &'a str) -> &'a str {
    if code.is_empty() { &"-" } else { &code }
}

fn gbc_flag_display(gbc_flag: GBCFlag) -> &'static str {
    match gbc_flag {
        GBCFlag::GBOnly => "GB only",
        GBCFlag::GBCAndGB => "GB & GBC",
        GBCFlag::GBCOnly => "GBC only",
    }
}

fn sgb_flag_display(sgb_flag: SGBFlag) -> &'static str {
    match sgb_flag {
        SGBFlag::NoSGB => "No support",
        SGBFlag::SGBSupport => "Supported",
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

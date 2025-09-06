use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    symbols::bar::NINE_LEVELS,
    widgets::{
        Block, BorderType, List, ListState, Paragraph, Row, StatefulWidget, Table, Widget,
        WidgetRef,
    },
};

use mule_gb::{DestinationCode, GBBinary, GBCFlag, RAMSize, ROMSize, SGBFlag, num_banks};
use psy::dasm::gb;

use crate::{
    InteractiveCommand,
    hex::Hex,
    view::{style_focus, style_normal},
};

#[derive(PartialEq, Clone, Copy)]
enum Focus {
    None,
    Restarts,
    Interrupts,
    Header,
    Banks,
}

static FOCUS_CYCLE_ORDER: [Focus; 4] = [
    Focus::Restarts,
    Focus::Interrupts,
    Focus::Header,
    Focus::Banks,
];

/// Cached disassembles that are only computed once
struct GBDisassembles {
    entry_point: Vec<String>,

    interrupt_v_blank: Vec<String>,
    interrupt_lcd_stat: Vec<String>,
    interrupt_timer: Vec<String>,
    interrupt_serial: Vec<String>,
    interrupt_joypad: Vec<String>,
    rst_0: Vec<String>,
    rst_1: Vec<String>,
    rst_2: Vec<String>,
    rst_3: Vec<String>,
    rst_4: Vec<String>,
    rst_5: Vec<String>,
    rst_6: Vec<String>,
    rst_7: Vec<String>,
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

        GBInteractiveState {
            bank_list_state,
            previous_focus: Focus::None,
            focus_on: Focus::Header,
            disassembles: GBDisassembles {
                entry_point: disassemble(&binary.header.entry_point),
                interrupt_v_blank: disassemble(&binary.interrupts.v_blank),
                interrupt_lcd_stat: disassemble(&binary.interrupts.lcd_stat),
                interrupt_timer: disassemble(&binary.interrupts.timer),
                interrupt_serial: disassemble(&binary.interrupts.serial),
                interrupt_joypad: disassemble(&binary.interrupts.joypad),
                rst_0: disassemble(&binary.restart_calls.rst_0),
                rst_1: disassemble(&binary.restart_calls.rst_1),
                rst_2: disassemble(&binary.restart_calls.rst_2),
                rst_3: disassemble(&binary.restart_calls.rst_3),
                rst_4: disassemble(&binary.restart_calls.rst_4),
                rst_5: disassemble(&binary.restart_calls.rst_5),
                rst_6: disassemble(&binary.restart_calls.rst_6),
                rst_7: disassemble(&binary.restart_calls.rst_7),
            },
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

fn disassemble(data: &[u8]) -> Vec<String> {
    match gb::disassemble(data) {
        Err(err) => vec![format!("Err disassemble: {}", err)],
        Ok(dis) => dis,
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
            Focus::Restarts => self.render_restart_detail(detail_block, content_detail, buf),
            Focus::Interrupts => self.render_interrupt_detail(detail_block, content_detail, buf),
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

    fn render_restart_detail(&self, block: Block, content_detail: Rect, buf: &mut Buffer) {
        let rst_0 = self.state.disassembles.rst_0.join("");
        let rst_1 = self.state.disassembles.rst_1.join("");
        let rst_2 = self.state.disassembles.rst_2.join("");
        let rst_3 = self.state.disassembles.rst_3.join("");
        let rst_4 = self.state.disassembles.rst_4.join("");
        let rst_5 = self.state.disassembles.rst_5.join("");
        let rst_6 = self.state.disassembles.rst_6.join("");
        let rst_7 = self.state.disassembles.rst_7.join("");
        let rows = [
            Row::new(vec!["RST 0:", &rst_0]),
            Row::new(vec!["RST 1:", &rst_1]),
            Row::new(vec!["RST 2:", &rst_2]),
            Row::new(vec!["RST 3:", &rst_3]),
            Row::new(vec!["RST 4:", &rst_4]),
            Row::new(vec!["RST 5:", &rst_5]),
            Row::new(vec!["RST 6:", &rst_6]),
            Row::new(vec!["RST 7:", &rst_7]),
        ];

        let widths = [Constraint::Length(7), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(block);
        Widget::render(table, content_detail, buf);
    }

    fn render_interrupt_detail(&self, block: Block, content_detail: Rect, buf: &mut Buffer) {
        let v_blank = self.state.disassembles.interrupt_v_blank.join("");
        let lcd_stat = self.state.disassembles.interrupt_lcd_stat.join("");
        let timer = self.state.disassembles.interrupt_timer.join("");
        let serial = self.state.disassembles.interrupt_serial.join("");
        let joypad = self.state.disassembles.interrupt_joypad.join("");
        let rows = [
            Row::new(vec!["V-Blank:", &v_blank]),
            Row::new(vec!["LCD-Stat:", &lcd_stat]),
            Row::new(vec!["Timer:", &timer]),
            Row::new(vec!["Serial:", &serial]),
            Row::new(vec!["Joypad:", &joypad]),
        ];

        let widths = [Constraint::Length(10), Constraint::Fill(1)];
        let table = Table::new(rows, widths).block(block);
        Widget::render(table, content_detail, buf);
    }

    fn render_header_detail(&self, block: Block, content_detail: Rect, buf: &mut Buffer) {
        let entry_text = self.state.disassembles.entry_point.join("");
        let logo_row_0_text = &logo_row(0, &self.gb_binary.header.logo_data);
        let logo_row_1_text = &logo_row(1, &self.gb_binary.header.logo_data);
        let logo_row_2_text = &logo_row(2, &self.gb_binary.header.logo_data);
        let logo_row_3_text = &logo_row(3, &self.gb_binary.header.logo_data);
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
            Row::new(vec!["Logo:", logo_row_0_text]),
            Row::new(vec!["     ", logo_row_1_text]),
            Row::new(vec!["     ", logo_row_2_text]),
            Row::new(vec!["     ", logo_row_3_text]),
            Row::new(vec!["", ""]),
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

fn logo_row(row: usize, logo_data: &[u8]) -> String {
    let mut result = String::new();
    let dis = row % 2;
    let offset = if row >= 2 { 24 } else { 0 };
    for i in (0..24).step_by(2) {
        let b = logo_data[offset + i + dis];
        let l0 = (b & 0xF0) >> 4;
        let l1 = b & 0xF;
        for s in (0..2).rev() {
            let mask = 0b11 << (s * 2);
            let l0_r = (l0 & mask) >> (s * 2);
            let l1_r = (l1 & mask) >> (s * 2);
            result.push(pixel_char(l0_r, l1_r))
        }
    }

    result
}

fn pixel_char(l0: u8, l1: u8) -> char {
    if l0 == 0b11 && l1 == 0b11 {
        '\u{2588}'
    } else if l0 == 0b11 && l1 == 0b01 {
        '\u{259C}'
    } else if l0 == 0b11 && l1 == 0b10 {
        '\u{259B}'
    } else if l0 == 0b11 && l1 == 0b00 {
        '\u{2580}'
    } else if l0 == 0b10 && l1 == 0b11 {
        '\u{2599}'
    } else if l0 == 0b10 && l1 == 0b01 {
        '\u{259A}'
    } else if l0 == 0b10 && l1 == 0b10 {
        '\u{258C}'
    } else if l0 == 0b10 && l1 == 0b00 {
        '\u{2598}'
    } else if l0 == 0b01 && l1 == 0b11 {
        '\u{259F}'
    } else if l0 == 0b01 && l1 == 0b01 {
        '\u{2590}'
    } else if l0 == 0b01 && l1 == 0b10 {
        '\u{259E}'
    } else if l0 == 0b01 && l1 == 0b00 {
        '\u{259D}'
    } else if l0 == 0b00 && l1 == 0b11 {
        '\u{2584}'
    } else if l0 == 0b00 && l1 == 0b01 {
        '\u{2597}'
    } else if l0 == 0b00 && l1 == 0b10 {
        '\u{2596}'
    } else if l0 == 0b00 && l1 == 0b00 {
        ' '
    } else {
        panic!("illegal combination: {:x} {:x}", l0, l1)
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

        let file_layout = Layout::vertical([
            Constraint::Max(3),
            Constraint::Max(3),
            Constraint::Max(3),
            Constraint::Fill(1),
        ]);
        let [gb_restarts, gb_interrupts, gb_header, gb_banks] = file_layout.areas(content_file);

        let restart_block = Block::bordered()
            .border_type(BorderType::Plain)
            .style(self.focus_style(Focus::Restarts))
            .title("Restart Calls");
        Paragraph::new(format!(
            "Non-default restarts: {}",
            non_default_restarts(self.gb_binary)
        ))
        .block(restart_block)
        .render(gb_restarts, buf);

        let interrupt_block = Block::bordered()
            .border_type(BorderType::Plain)
            .style(self.focus_style(Focus::Interrupts))
            .title("Interrupts");
        Paragraph::new(format!(
            "Non-Default interrupts: {}",
            non_default_interrupts(self.gb_binary)
        ))
        .block(interrupt_block)
        .render(gb_interrupts, buf);

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

fn non_default_restarts(binary: &GBBinary) -> usize {
    let mut n = 0;
    if !default_vector(&binary.restart_calls.rst_0) {
        n += 1
    }
    if !default_vector(&binary.restart_calls.rst_1) {
        n += 1
    }
    if !default_vector(&binary.restart_calls.rst_2) {
        n += 1
    }
    if !default_vector(&binary.restart_calls.rst_3) {
        n += 1
    }
    if !default_vector(&binary.restart_calls.rst_4) {
        n += 1
    }
    if !default_vector(&binary.restart_calls.rst_5) {
        n += 1
    }
    if !default_vector(&binary.restart_calls.rst_6) {
        n += 1
    }
    if !default_vector(&binary.restart_calls.rst_7) {
        n += 1
    }
    n
}

fn non_default_interrupts(binary: &GBBinary) -> usize {
    let mut n = 0;
    if !default_vector(&binary.interrupts.v_blank) {
        n += 1;
    }
    if !default_vector(&binary.interrupts.lcd_stat) {
        n += 1;
    }
    if !default_vector(&binary.interrupts.timer) {
        n += 1;
    }
    if !default_vector(&binary.interrupts.serial) {
        n += 1;
    }
    if !default_vector(&binary.interrupts.joypad) {
        n += 1;
    }
    n
}

fn default_vector(data: &[u8]) -> bool {
    for i in 0..data.len() {
        if data[i] != 0xFF {
            return false;
        }
    }
    true
}

fn bank_list(binary: &GBBinary) -> Vec<String> {
    let n = num_banks(binary.header.rom_size);
    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        result.push(format!("Bank {}", i));
    }
    result
}

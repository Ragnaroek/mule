use std::collections::HashMap;

use crate::{
    hex::{GBDisassembly, HexWidget},
    toggle::toggle,
    view::{BinaryViewWidget, TileWidget},
};
use egui::{Frame, Grid, Margin};
use mule_gb::{self, DestinationCode, GBBinary, GBCFlag, RAMSize, ROMSize, SGBFlag};
use psy::dasm::gb;

const SIDE_SEG_MARGIN: i8 = 8;

struct BankViewState {
    bank: usize,
    disassemble: Option<GBDisassembly>,
    hex: HexWidget,
}

pub struct GBViewWidget {
    binary: GBBinary,
    binary_disassemble: GBBinaryDisassembled,

    tile_restarts: TileWidget,
    tile_interrupts: TileWidget,
    tile_header: TileWidget,
    tile_banks: TileWidget,

    selected: GBSelected,
    bank_view_state: Option<BankViewState>,
}

// contains everything that is only computed once from the GBBinary
// TOOD rename to GBBinaryHeaderDisassembly
struct GBBinaryDisassembled {
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
    entry_point: Vec<String>,
}

#[derive(PartialEq)]
enum GBSelected {
    Restarts,
    Interrupts,
    Header,
    Banks(usize),
}

impl GBViewWidget {
    pub fn new(binary: GBBinary) -> GBViewWidget {
        let mut tile_banks = TileWidget::new(format!("Banks ({})", binary.bank_data.len()));
        tile_banks.not_selectable();

        let binary_disassemble = initial_disassemble(&binary);

        GBViewWidget {
            binary,
            binary_disassemble,

            tile_restarts: TileWidget::new("Restart Calls".to_string()),
            tile_interrupts: TileWidget::new("Interrupts".to_string()),
            tile_header: TileWidget::new("Header".to_string()),
            tile_banks,

            selected: GBSelected::Header,
            bank_view_state: None,
        }
    }

    fn render_header(&self, ui: &mut egui::Ui) {
        Grid::new("header_grid")
            .spacing([40.0, 3.0])
            .show(ui, |ui| {
                ui.label("Logo:");
                ui.label("TODO");
                ui.end_row();

                ui.label("Entry Point:");
                ui.label(self.binary_disassemble.entry_point.join(""));
                ui.end_row();

                ui.label("Game Title:");
                ui.label(&self.binary.header.game_title);
                ui.end_row();

                ui.label("Manufacturer Code:");
                ui.label(manufacturer_display(&self.binary.header.manufacturer_code));
                ui.end_row();

                ui.label("GBC Flag:");
                ui.label(gbc_flag_display(self.binary.header.gbc_flag));
                ui.end_row();

                ui.label("Licensee Code:");
                ui.label(&format!("{:?}", self.binary.header.licensee_code));
                ui.end_row();

                ui.label("Super Gameboy Flag:");
                ui.label(sgb_flag_display(self.binary.header.sgb_flag));
                ui.end_row();

                ui.label("Cartridge Type:");
                ui.label(&format!("{:?}", self.binary.header.cartridge_type));
                ui.end_row();

                ui.label("ROM Size:");
                ui.label(rom_display(self.binary.header.rom_size));
                ui.end_row();

                ui.label("RAM Size:");
                ui.label(ram_display(self.binary.header.ram_size));
                ui.end_row();

                ui.label("Destination Code:");
                ui.label(dest_code_display(self.binary.header.destination_code));
                ui.end_row();

                ui.label("ROM Version:");
                ui.label(&format!("{}", self.binary.header.rom_version));
                ui.end_row();

                ui.label("Checksum:");
                ui.label(&format!("{}", self.binary.header.checksum));
                ui.end_row();

                ui.label("Global Checksum:");
                ui.label(&format!("{}", self.binary.header.global_checksum));
                ui.end_row();
            });
    }
}

fn initial_disassemble(binary: &GBBinary) -> GBBinaryDisassembled {
    GBBinaryDisassembled {
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
        entry_point: disassemble(&binary.header.entry_point),
    }
}

fn disassemble(data: &[u8]) -> Vec<String> {
    match gb::disassemble(data) {
        Err(err) => vec![format!("Err disassemble: {}", err)],
        Ok(dis) => dis,
    }
}

impl BinaryViewWidget for GBViewWidget {
    fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("master_panel")
            .resizable(true)
            .default_size(300.0)
            .frame(Frame::new().inner_margin(Margin::same(SIDE_SEG_MARGIN)))
            .show_inside(ui, |ui| {
                self.tile_restarts
                    .set_selected(self.selected == GBSelected::Restarts);
                if self
                    .tile_restarts
                    .show(ui, |ui| {
                        ui.label(&format!(
                            "Non-default restarts: {}",
                            non_default_restarts(&self.binary)
                        ));
                    })
                    .clicked()
                {
                    self.selected = GBSelected::Restarts;
                };
                ui.add_space(SIDE_SEG_MARGIN as f32);

                self.tile_interrupts
                    .set_selected(self.selected == GBSelected::Interrupts);
                if self
                    .tile_interrupts
                    .show(ui, |ui| {
                        ui.label(&format!(
                            "Non-default interrupts: {}",
                            non_default_interrupts(&self.binary)
                        ));
                    })
                    .clicked()
                {
                    self.selected = GBSelected::Interrupts;
                };
                ui.add_space(SIDE_SEG_MARGIN as f32);

                self.tile_header
                    .set_selected(self.selected == GBSelected::Header);
                if self
                    .tile_header
                    .show(ui, |ui| {
                        ui.label(&format!(
                            "title:{} | type:{:?}",
                            self.binary.header.game_title, self.binary.header.cartridge_type
                        ));
                    })
                    .clicked()
                {
                    self.selected = GBSelected::Header;
                };
                ui.add_space(SIDE_SEG_MARGIN as f32);

                if let GBSelected::Banks(_) = self.selected {
                    self.tile_banks.set_selected(true);
                } else {
                    self.tile_banks.set_selected(false);
                }
                self.tile_banks.show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for bank in 0..self.binary.bank_data.len() {
                            let selected = match &self.selected {
                                GBSelected::Banks(bank_num) => *bank_num == bank,
                                _ => false,
                            };

                            if ui
                                .selectable_label(selected, format!("Bank {}", bank))
                                .clicked()
                            {
                                self.selected = GBSelected::Banks(bank);

                                let switch_bank = if let Some(state) = &self.bank_view_state {
                                    state.bank != bank
                                } else {
                                    true
                                };
                                if switch_bank {
                                    self.bank_view_state = Some(BankViewState {
                                        disassemble: None,
                                        bank,
                                        hex: HexWidget::new(self.binary.bank_data[bank].clone()),
                                    });
                                }
                            };
                        }
                    });
                })
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if self.tile_banks.is_selected() {
                if let Some(bank_state) = &mut self.bank_view_state {
                    render_bank_view(ui, bank_state);
                }
            } else if self.tile_header.is_selected() {
                self.render_header(ui);
            } else if self.tile_interrupts.is_selected() {
                ui.label(format!(
                    "V-Blank: {}",
                    self.binary_disassemble.interrupt_v_blank.join("")
                ));
                ui.label(format!(
                    "LCD-Stat: {}",
                    self.binary_disassemble.interrupt_lcd_stat.join("")
                ));
                ui.label(format!(
                    "Timer: {}",
                    self.binary_disassemble.interrupt_timer.join("")
                ));
                ui.label(format!(
                    "Serial: {}",
                    self.binary_disassemble.interrupt_serial.join("")
                ));
                ui.label(format!(
                    "V-Blank: {}",
                    self.binary_disassemble.interrupt_joypad.join("")
                ));
            } else {
                ui.label(format!("RST 0: {}", self.binary_disassemble.rst_0.join("")));
                ui.label(format!("RST 1: {}", self.binary_disassemble.rst_1.join("")));
                ui.label(format!("RST 2: {}", self.binary_disassemble.rst_2.join("")));
                ui.label(format!("RST 3: {}", self.binary_disassemble.rst_3.join("")));
                ui.label(format!("RST 4: {}", self.binary_disassemble.rst_4.join("")));
                ui.label(format!("RST 5: {}", self.binary_disassemble.rst_5.join("")));
                ui.label(format!("RST 6: {}", self.binary_disassemble.rst_6.join("")));
                ui.label(format!("RST 7: {}", self.binary_disassemble.rst_7.join("")));
            };
        });
    }
}

fn render_bank_view(ui: &mut egui::Ui, bank_state: &mut BankViewState) {
    ui.horizontal(|ui| {
        let toggle_state_before = bank_state.disassemble.is_some();
        let mut toggle_state = toggle_state_before;
        ui.add(toggle(&mut toggle_state));

        if toggle_state != toggle_state_before {
            if toggle_state && bank_state.disassemble.is_none() {
                bank_state.disassemble = Some(GBDisassembly {
                    instructions: HashMap::new(),
                });
            } else if !toggle_state {
                bank_state.disassemble = None;
            }
        }

        ui.label("Show Disassemble");
    });
    bank_state.hex.show(ui, bank_state.disassemble.as_ref());
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

fn dest_code_display(dest_code: DestinationCode) -> &'static str {
    match dest_code {
        DestinationCode::Japanese => "Japanese",
        DestinationCode::NonJapanese => "No Japanese",
    }
}

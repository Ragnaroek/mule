use crate::view::{BinaryViewWidget, TileWidget};
use egui::{Frame, Margin};
use mule_gb::{self, GBBinary};
use psy::dasm::gb;

const SIDE_SEG_MARGIN: i8 = 8;

pub struct GBViewWidget {
    binary: GBBinary,
    binary_disassemble: GBBinaryDisassembled,

    tile_restarts: TileWidget,
    tile_interrupts: TileWidget,
    tile_header: TileWidget,
    tile_banks: TileWidget,

    selected: GBSelected,
}

// contains everything that is only computed once from the GBBinary
struct GBBinaryDisassembled {
    rst_0: Vec<String>,
    rst_1: Vec<String>,
    rst_2: Vec<String>,
    rst_3: Vec<String>,
    rst_4: Vec<String>,
    rst_5: Vec<String>,
    rst_6: Vec<String>,
    rst_7: Vec<String>,
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
        }
    }
}

fn initial_disassemble(binary: &GBBinary) -> GBBinaryDisassembled {
    GBBinaryDisassembled {
        rst_0: disassemble(&binary.restart_calls.rst_0),
        rst_1: disassemble(&binary.restart_calls.rst_1),
        rst_2: disassemble(&binary.restart_calls.rst_2),
        rst_3: disassemble(&binary.restart_calls.rst_3),
        rst_4: disassemble(&binary.restart_calls.rst_4),
        rst_5: disassemble(&binary.restart_calls.rst_5),
        rst_6: disassemble(&binary.restart_calls.rst_6),
        rst_7: disassemble(&binary.restart_calls.rst_7),
    }
}

fn disassemble(data: &[u8]) -> Vec<String> {
    match gb::disassemble(data) {
        Err(err) => vec![format!("Err disassemble: {}", err)],
        Ok(dis) => dis,
    }
}

impl BinaryViewWidget for GBViewWidget {
    fn show(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("master_panel")
            .resizable(true)
            .default_width(300.0)
            .frame(Frame::new().inner_margin(Margin::same(SIDE_SEG_MARGIN)))
            .show(ctx, |ui| {
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
                        for i in 0..self.binary.bank_data.len() {
                            let selected = match &self.selected {
                                GBSelected::Banks(bank_num) => *bank_num == i,
                                _ => false,
                            };

                            if ui
                                .selectable_label(selected, format!("Bank {}", i))
                                .clicked()
                            {
                                self.selected = GBSelected::Banks(i);
                            };
                        }
                    });
                })
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.tile_banks.is_selected() {
                ()
            } else if self.tile_header.is_selected() {
                ()
            } else if self.tile_interrupts.is_selected() {
                ()
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

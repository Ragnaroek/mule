use crate::view::{BinaryViewWidget, TileWidget};
use egui::{Frame, Margin};
use mule_gb::{self, GBBinary};

const SIDE_SEG_MARGIN: i8 = 8;

pub struct GBViewWidget {
    binary: GBBinary,
    tile_restarts: TileWidget,
    tile_interrupts: TileWidget,
    tile_header: TileWidget,
    tile_banks: TileWidget,

    selected: GBSelected,
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

        GBViewWidget {
            binary,
            tile_restarts: TileWidget::new("Restart Calls".to_string()),
            tile_interrupts: TileWidget::new("Interrupts".to_string()),
            tile_header: TileWidget::new("Header".to_string()),
            tile_banks,

            selected: GBSelected::Header,
        }
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
                        ui.label("Non-default restarts: 4");
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
                        ui.label("Non-default interrupts: 4");
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
                        ui.label("Non-default interrupts: 4");
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
            let selected_debug = if self.tile_banks.is_selected() {
                "banks"
            } else if self.tile_header.is_selected() {
                "header"
            } else if self.tile_interrupts.is_selected() {
                "interrupts"
            } else {
                "restarts"
            };

            ui.heading(selected_debug);
            ui.separator();
        });
    }
}

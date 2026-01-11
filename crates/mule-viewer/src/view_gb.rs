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
}

impl GBViewWidget {
    pub fn new(binary: GBBinary) -> GBViewWidget {
        GBViewWidget {
            binary,
            tile_restarts: TileWidget::new("Restart Calls".to_string()),
            tile_interrupts: TileWidget::new("Interrupts".to_string()),
            tile_header: TileWidget::new("Header".to_string()),
            tile_banks: TileWidget::new("Banks (xxx)".to_string()),
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
                if self
                    .tile_restarts
                    .show(ui, |ui| {
                        ui.label("Non-default restarts: 4");
                    })
                    .clicked()
                {
                    //TODO impl TileGroup to manage this better
                    self.tile_restarts.set_selected(false);
                    self.tile_interrupts.set_selected(false);
                    self.tile_header.set_selected(false);
                    self.tile_banks.set_selected(false);

                    self.tile_restarts.set_selected(true);
                };
                ui.add_space(SIDE_SEG_MARGIN as f32);

                if self
                    .tile_interrupts
                    .show(ui, |ui| {
                        ui.label("Non-default interrupts: 4");
                    })
                    .clicked()
                {
                    self.tile_restarts.set_selected(false);
                    self.tile_interrupts.set_selected(false);
                    self.tile_header.set_selected(false);
                    self.tile_banks.set_selected(false);

                    self.tile_interrupts.set_selected(true);
                };
                ui.add_space(SIDE_SEG_MARGIN as f32);

                if self
                    .tile_header
                    .show(ui, |ui| {
                        ui.label("Non-default interrupts: 4");
                    })
                    .clicked()
                {
                    self.tile_restarts.set_selected(false);
                    self.tile_interrupts.set_selected(false);
                    self.tile_header.set_selected(false);
                    self.tile_banks.set_selected(false);

                    self.tile_header.set_selected(true);
                };
                ui.add_space(SIDE_SEG_MARGIN as f32);

                if self
                    .tile_banks
                    .show(ui, |ui| {
                        egui::CollapsingHeader::new("Bank 0")
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label("...");
                            });

                        egui::CollapsingHeader::new("Bank 1").show(ui, |ui| {
                            ui.label("...");
                        });
                    })
                    .clicked()
                {
                    self.tile_restarts.set_selected(false);
                    self.tile_interrupts.set_selected(false);
                    self.tile_header.set_selected(false);
                    self.tile_banks.set_selected(false);

                    self.tile_banks.set_selected(true);
                };
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

use egui::{Frame, Margin};
use mule_elf::ElfBinary;

use crate::view::{BinaryViewWidget, TileWidget};

const SIDE_SEG_MARGIN: i8 = 8;

pub struct ElfViewWidget {
    binary: ElfBinary,

    tile_header: TileWidget,
}

impl ElfViewWidget {
    pub fn new(binary: ElfBinary) -> ElfViewWidget {
        ElfViewWidget {
            binary,

            tile_header: TileWidget::new("Header".to_string()),
        }
    }
}

impl BinaryViewWidget for ElfViewWidget {
    fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("master_panel")
            .resizable(true)
            .default_size(300.0)
            .frame(Frame::new().inner_margin(Margin::same(SIDE_SEG_MARGIN)))
            .show(ui, |ui| {
                self.tile_header.show(ui, |ui| {
                    ui.label(&format!("elf summary?"));
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {});
    }
}

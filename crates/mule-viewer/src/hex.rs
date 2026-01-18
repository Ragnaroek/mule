use std::f32;

use egui::{FontId, RichText, ScrollArea, TextEdit};

pub struct HexWidget<'a> {
    data: &'a [u8],
}

impl<'a> HexWidget<'a> {
    pub fn new(data: &'a [u8]) -> HexWidget<'a> {
        HexWidget { data }
    }
    pub fn show(&self, ui: &mut egui::Ui) {
        let font_id = FontId::new(14.0, egui::FontFamily::Monospace);
        let hex_block_width = ui.fonts_mut(|fonts| fonts.glyph_width(&font_id, 'A')) * 9.0; // pixel width for 4-byte block like "C30C02CD ", including the padding at the end

        ScrollArea::vertical().show(ui, |ui| {
            let mut offset = 0;
            while offset < self.data.len() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&format!("{:04X}", offset)).font(font_id.clone()));
                    ui.add_space(10.0);
                    let num_hex_blocks = (ui.available_width() / hex_block_width).floor() as usize;
                    let mut hex_string = String::new();
                    for _ in 0..num_hex_blocks {
                        hex_string.push_str(&format_block(self.data, offset));
                        hex_string.push(' ');
                        offset += 4;
                    }
                    ui.add(
                        TextEdit::singleline(&mut hex_string)
                            .desired_width(f32::INFINITY)
                            .font(font_id.clone()),
                    );
                });
            }
        });
    }
}

fn format_block(data: &[u8], offset: usize) -> String {
    let mut block_str = String::new();
    for i in 0..4 {
        if (offset + i) >= data.len() {
            break;
        }
        block_str.push_str(&format!("{:02X}", data[offset + i]));
    }
    block_str
}

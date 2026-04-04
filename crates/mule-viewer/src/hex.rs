use egui::{FontId, Pos2, Rect, RichText, ScrollArea};
use psy::dasm::gb::GBDisassembly;

const GUTTER_SPACE_HEX: &str = "  ";
const GUTTER_SPACE_DIS: &str = "            "; // empty line number + double because font-size is half
const RIGHT_SPACING: f32 = 15.0;

struct DisassembleOverlay {
    pos: Pos2,
    text: String,
}

pub struct HexWidget {
    data: Vec<u8>,
    byte_selected: usize,
    disassemble_overlay: Option<DisassembleOverlay>,
}

impl HexWidget {
    pub fn new(data: Vec<u8>) -> HexWidget {
        HexWidget {
            data,
            byte_selected: 0,
            disassemble_overlay: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, show_disassemble: Option<&GBDisassembly>) {
        let font_id = FontId::new(14.0, egui::FontFamily::Monospace);
        let char_width = ui.fonts_mut(|fonts| fonts.glyph_width(&font_id, 'A'));
        let hex_block_width = char_width * 9.0; // pixel width for 4-byte block like "C30C02CD ", including the padding at the end
        let gutter_width = char_width * (4.0 + GUTTER_SPACE_HEX.chars().count() as f32);
        let num_hex_blocks = ((ui.available_width() - gutter_width - RIGHT_SPACING)
            / hex_block_width)
            .floor() as usize;
        let num_row_bytes = num_hex_blocks * 4;

        ScrollArea::vertical().show(ui, |ui| {
            if let Some(dis) = show_disassemble {
                self.render_disassemble(ui, char_width, gutter_width, dis);
            } else {
                self.render_hex_only(ui, font_id, num_row_bytes);
            }
        });

        if let Some(overlay) = &self.disassemble_overlay {
            let area = egui::Area::new("overlay_area".into())
                .fixed_pos(overlay.pos)
                .order(egui::Order::Foreground);

            let text = overlay.text.clone();
            area.show(ui.ctx(), |ui| {
                egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
                    ui.label(text);
                    if ui.button("Close").clicked() {
                        self.disassemble_overlay = None
                    }
                });
            });
        }
    }

    fn render_hex_only(&self, ui: &mut egui::Ui, font_id: FontId, num_row_bytes: usize) {
        let mut offset = 0;
        while offset < self.data.len() {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;

                let mut row_string = String::new();
                row_string.push_str(&format!("{:04X}", offset));
                row_string.push_str("  ");

                for i in 0..num_row_bytes {
                    let data_ix = offset + i;
                    if data_ix >= self.data.len() {
                        break;
                    }
                    row_string.push_str(&format!("{:02X}", self.data[data_ix]));
                    if (data_ix + 1) % 4 == 0 {
                        row_string.push(' ');
                    }
                }

                ui.label(RichText::new(row_string).font(font_id.clone()));
            });

            offset += num_row_bytes;
        }
    }

    fn render_disassemble(
        &self,
        ui: &mut egui::Ui,
        char_width: f32,
        gutter_width: f32,
        dis: &GBDisassembly,
    ) {
        let font_id_hex = FontId::new(14.0, egui::FontFamily::Monospace);
        let font_id_dis = FontId::new(7.0, egui::FontFamily::Monospace);
        let max_block_width = char_width * 3.0; // 'FF ' (one byte hex + one spacing, worst-case can be optimized if computed per line)
        let num_row_bytes = ((ui.available_width() - gutter_width - RIGHT_SPACING)
            / max_block_width)
            .floor() as usize;

        let mut offset = 0;
        let mut line_start_offset = offset;
        let mut line_end_bytes = num_row_bytes;
        let mut rects: Vec<(usize, usize)> = Vec::new();

        let mut row_string = String::new();
        let mut dis_string = String::new();
        row_string.push_str(&format!("{:04X}", line_start_offset));
        row_string.push_str(GUTTER_SPACE_HEX);
        dis_string.push_str(GUTTER_SPACE_DIS);
        for i in 0..dis.instructions.len() {
            let instr = &dis.instructions[i];
            if offset + instr.len > line_end_bytes {
                let r = ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(row_string.clone()).font(font_id_hex.clone()));
                        ui.label(RichText::new(dis_string.clone()).font(font_id_dis.clone()));
                    })
                });

                let padding = char_width / 3.0;
                for rect in &rects {
                    let bx = Rect::from_min_max(
                        Pos2 {
                            x: r.response.rect.min.x + rect.0 as f32 * char_width - padding,
                            y: r.response.rect.min.y,
                        },
                        Pos2 {
                            x: r.response.rect.min.x + rect.1 as f32 * char_width + padding,
                            y: r.response.rect.max.y,
                        },
                    );
                    ui.painter().rect_stroke(
                        bx,
                        0.0,
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                        egui::StrokeKind::Outside,
                    );
                }

                line_start_offset = offset;
                line_end_bytes += num_row_bytes;
                row_string.clear();
                dis_string.clear();
                rects.clear();
                row_string.push_str(&format!("{:04X}", line_start_offset));
                row_string.push_str(GUTTER_SPACE_HEX);
                dis_string.push_str(GUTTER_SPACE_DIS);
            }

            let start = row_string.chars().count();
            for _ in 0..instr.len {
                row_string.push_str(&format!("{:02X}", self.data[offset]));
                offset += 1;
            }
            let max = instr.len * 2 * 2; //2 chars per byte + 2 times the width because font size is half
            let truncated = instr.instr.mnemonic.chars().take(max).collect::<String>();
            dis_string.push_str(&format!("{:<max$}  ", truncated));

            rects.push((start, row_string.chars().count()));

            row_string.push(' ');
        }
    }
}

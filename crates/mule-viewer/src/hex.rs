use egui::{Color32, FontId, Pos2, Rect, RichText, ScrollArea};

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
    pub fn show(&mut self, ui: &mut egui::Ui) {
        let font_id = FontId::new(14.0, egui::FontFamily::Monospace);
        let hex_block_width = ui.fonts_mut(|fonts| fonts.glyph_width(&font_id, 'A')) * 9.0; // pixel width for 4-byte block like "C30C02CD ", including the padding at the end

        ScrollArea::vertical().show(ui, |ui| {
            let mut offset = 0;
            let mut selected_pos: Option<Rect> = None;
            while offset < self.data.len() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&format!("{:04X}", offset)).font(font_id.clone()));
                    ui.add_space(10.0);
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let num_hex_blocks = (ui.available_width() / hex_block_width).floor() as usize;
                    for _ in 0..num_hex_blocks {
                        for i in 0..4 {
                            if (offset + i) >= self.data.len() {
                                break;
                            }
                            let byte_text =
                                RichText::new(&format!("{:02X}", self.data[offset + i]))
                                    .font(font_id.clone());
                            let byte_text = if offset == self.byte_selected {
                                byte_text.background_color(Color32::WHITE)
                            } else {
                                byte_text
                            };
                            let resp = ui.label(byte_text);
                            if offset == self.byte_selected {
                                selected_pos = Some(resp.rect)
                            }

                            offset += 1;
                        }
                        ui.label(RichText::new(" ").font(font_id.clone()));
                    }
                });
            }

            if ui.input(|i| i.key_pressed(egui::Key::D)) {
                ui.ctx()
                    .input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::D));
                log::debug!("d pressed {:?}", selected_pos);
                if let Some(rect) = selected_pos {
                    let dis_result = psy::dasm::gb::disassemble(
                        &self.data[self.byte_selected
                            ..(self.byte_selected + psy::arch::sm83::MAX_INSTRUCTION_BYTE_LENGTH)],
                    );

                    let text = if let Ok(dis) = dis_result {
                        dis[0].to_string()
                    } else {
                        "ERR".to_string()
                    };

                    let pos = Pos2::new(rect.min.x, rect.max.y);
                    self.disassemble_overlay = Some(DisassembleOverlay { pos, text });
                }
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
}

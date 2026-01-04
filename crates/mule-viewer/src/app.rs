use eframe::egui;
use egui::{ColorImage, TextureHandle, load::SizedTexture};

use crate::util::execute_async;

pub struct MuleApp {
    logo: TextureHandle,
}

impl MuleApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> MuleApp {
        let logo_bytes = include_bytes!("../assets/logo.png");
        let image = image::load_from_memory(logo_bytes).unwrap().to_rgba8();
        let logo_size = [image.width() as usize, image.height() as usize];
        let pixels = image.into_raw();
        let color_image = ColorImage::from_rgba_unmultiplied(logo_size, &pixels);
        let logo = cc
            .egui_ctx
            .load_texture("logo", color_image, egui::TextureOptions::LINEAR);

        MuleApp { logo }
    }
}

impl eframe::App for MuleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let logo_size = self.logo.size_vec2();
                ui.add_space(ui.available_height() / 2.0 - logo_size.y);

                let texture = SizedTexture::new(self.logo.id(), logo_size);
                ui.add(egui::Image::new(texture));
                ui.add_space(20.0);
                ui.label("Upload your binary to start analysing");
                ui.add_space(5.0);
                if ui.button("Upload").clicked() {
                    execute_async(async move {
                        if let Some(file) = rfd::AsyncFileDialog::new().pick_file().await {
                            let bytes = file.read().await;
                            log::debug!("file bytes = {:?}", bytes);
                            let loaded = mule_gb::load(&bytes).unwrap();
                            log::debug!("parsed = {:?}", loaded.header.game_title);
                            //*loaded_file.lock().unwrap() = Some(bytes);
                            //ctx.request_repaint();
                        }
                    });
                }
            });
        });
    }
}

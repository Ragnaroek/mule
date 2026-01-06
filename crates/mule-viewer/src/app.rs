use eframe::egui;
use egui::{Button, ColorImage, TextureHandle, containers::menu::MenuButton, load::SizedTexture};
use mule_gb::GBBinary;
use poll_promise::Promise;
use std::path::PathBuf;

use crate::util::{FileUpload, open_file};

pub enum Binary {
    GB(GBBinary),
}

struct BinaryFile {
    name: String,
    binary: Binary,
}

pub struct MuleApp {
    logo: TextureHandle,

    binary_file_open_promise: Option<Promise<FileUpload>>,
    binary_file: Option<BinaryFile>,
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

        MuleApp {
            logo,
            binary_file_open_promise: None,
            binary_file: None,
        }
    }

    fn handle_file_upload(&mut self) {
        if let Some(binary_promise) = &self.binary_file_open_promise {
            if let Some(file_upload) = binary_promise.ready() {
                let parsed_file = mule_gb::load(&file_upload.bytes).expect("gb file binary parse");
                self.binary_file = Some(BinaryFile {
                    name: file_upload.name.clone(),
                    binary: Binary::GB(parsed_file),
                });
                self.binary_file_open_promise = None;
            }
        }
    }
}

impl eframe::App for MuleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_file_upload();

        if let Some(binary_file) = &self.binary_file {
            let name = match &binary_file.binary {
                Binary::GB(gb_binary) => gb_binary.header.game_title.clone(),
                _ => "????".to_string(),
            };

            egui::TopBottomPanel::top("menu")
                .exact_height(20.0)
                .show(ctx, |ui| {
                    egui::MenuBar::new().ui(ui, |ui| {
                        ui.set_height(20.00);

                        MenuButton::from_button(Button::new("icon_button"))
                            .ui(ui, |ui| ui.heading("button"));

                        ui.heading("Top Menu");
                    });
                });

            egui::TopBottomPanel::top("binary_info")
                .exact_height(80.0)
                .show(ctx, |ui| {
                    ui.heading(format!(
                        "binary info {:?} + game name {}",
                        binary_file.name, name
                    ))
                });
            // MASTER
            egui::SidePanel::left("master_panel")
                .resizable(true)
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.heading("Items");
                    ui.separator();
                });

            // DETAIL
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Details");
                ui.separator();
            });
        } else {
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
                        let egui_ctx = ctx.clone();
                        self.binary_file_open_promise =
                            Some(poll_promise::Promise::spawn_local(async move {
                                let file_upload = open_file().await;
                                egui_ctx.request_repaint(); // Wake ui thread
                                file_upload
                            }));
                    }
                });
            });
        }
    }
}

use eframe::egui;
use egui::{
    Button, ColorImage, ScrollArea, TextureHandle, containers::menu::MenuButton, load::SizedTexture,
};
use mule_gb::GBBinary;
use poll_promise::Promise;

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
    logo_menu: TextureHandle,

    binary_file_open_promise: Option<Promise<FileUpload>>,
    binary_file: Option<BinaryFile>,
}

const MENU_HEIGHT: f32 = 35.0;

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

        let logo_bytes = include_bytes!("../assets/logo_menu.png");
        let image = image::load_from_memory(logo_bytes).unwrap().to_rgba8();
        let logo_size = [image.width() as usize, image.height() as usize];
        let pixels = image.into_raw();
        let color_image = ColorImage::from_rgba_unmultiplied(logo_size, &pixels);
        let logo_menu =
            cc.egui_ctx
                .load_texture("logo_menu", color_image, egui::TextureOptions::LINEAR);

        MuleApp {
            logo,
            logo_menu,
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

    fn show_start_screen(&mut self, ctx: &egui::Context) {
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

    fn show_top_menu(ctx: &egui::Context, binary_name: &str, logo: &TextureHandle) {
        egui::TopBottomPanel::top("menu")
            .exact_height(MENU_HEIGHT)
            .show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.set_max_height(MENU_HEIGHT);

                    let button = Button::image(logo);
                    MenuButton::from_button(button).ui(ui, |ui| {
                        ScrollArea::vertical()
                            .max_height(ui.ctx().content_rect().height() - 16.0)
                            .show(ui, |ui| {
                                ui.button("About").clicked();
                            });
                    });
                    ui.label(format!("Analyzing file: {}", binary_name));
                });
            });
    }
}

fn tile_frame(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .inner_margin(egui::Margin::same(6))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(title)
                    .monospace()
                    .strong()
                    .color(egui::Color32::LIGHT_YELLOW),
            );
            ui.separator();
            body(ui);
        });
}

impl eframe::App for MuleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_file_upload();

        if let Some(binary_file) = &self.binary_file {
            let name = match &binary_file.binary {
                Binary::GB(gb_binary) => gb_binary.header.game_title.clone(),
                _ => "????".to_string(),
            };

            MuleApp::show_top_menu(ctx, &binary_file.name, &self.logo_menu);

            // MASTER
            egui::SidePanel::left("master_panel")
                .resizable(true)
                .default_width(200.0)
                .show(ctx, |ui| {
                    tile_frame(ui, "Restart Calls", |ui| {
                        ui.label("Non-default restarts: 4");
                    });

                    tile_frame(ui, "Banks (2)", |ui| {
                        egui::CollapsingHeader::new("Bank 0")
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label("...");
                            });

                        egui::CollapsingHeader::new("Bank 1").show(ui, |ui| {
                            ui.label("...");
                        });
                    });
                });

            // DETAIL
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Details");
                ui.separator();
            });
        } else {
            self.show_start_screen(ctx);
        }
    }
}

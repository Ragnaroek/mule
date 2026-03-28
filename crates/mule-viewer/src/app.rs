use eframe::egui;
use egui::{
    Button, ColorImage, ScrollArea, TextureHandle, containers::menu::MenuButton, load::SizedTexture,
};
use poll_promise::Promise;

use crate::{
    util::{FileUpload, open_file},
    view::BinaryViewWidget,
    view_gb::GBViewWidget,
};

struct BinaryViewOpen {
    file_name: String,
    view: Box<dyn BinaryViewWidget>,
}

pub struct MuleApp {
    logo: TextureHandle,
    logo_menu: TextureHandle,

    binary_file_open_promise: Option<Promise<FileUpload>>,
    binary_view_open: Option<BinaryViewOpen>,
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

        #[cfg(feature = "debug")]
        let binary_view_open = {
            let debug_bytes = include_bytes!("../debug/debug_rom.gb");
            let gb_binary = mule_gb::load(debug_bytes).expect("debug gb file binary parse");
            Some(BinaryViewOpen {
                file_name: "debug_rom.gb".to_string(),
                view: Box::new(GBViewWidget::new(gb_binary)),
            })
        };

        #[cfg(not(feature = "debug"))]
        let binary_view_open = None;

        MuleApp {
            logo,
            logo_menu,
            binary_file_open_promise: None,
            binary_view_open,
        }
    }

    fn handle_file_upload(&mut self) {
        if let Some(binary_promise) = &self.binary_file_open_promise {
            if let Some(file_upload) = binary_promise.ready() {
                let gb_binary = mule_gb::load(&file_upload.bytes).expect("gb file binary parse");
                self.binary_view_open = Some(BinaryViewOpen {
                    file_name: file_upload.name.clone(),
                    view: Box::new(GBViewWidget::new(gb_binary)),
                });
                self.binary_file_open_promise = None;
            }
        }
    }

    fn show_start_screen(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                let logo_size = self.logo.size_vec2();
                ui.add_space(ui.available_height() / 2.0 - logo_size.y);

                let texture = SizedTexture::new(self.logo.id(), logo_size);
                ui.add(egui::Image::new(texture));
                ui.add_space(20.0);
                ui.label("Upload your binary to start analysing");
                ui.add_space(5.0);
                if ui.button("Upload").clicked() {
                    let egui_ctx = ui.ctx().clone();
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

    fn show_top_menu(ui: &mut egui::Ui, binary_name: &str, logo: &TextureHandle) {
        egui::Panel::top("menu")
            .exact_size(MENU_HEIGHT)
            .show_inside(ui, |ui| {
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

impl eframe::App for MuleApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_file_upload();

        if let Some(binary_view_open) = &mut self.binary_view_open {
            MuleApp::show_top_menu(ui, &binary_view_open.file_name, &self.logo_menu);
            binary_view_open.view.show(ui);
        } else {
            self.show_start_screen(ui);
        }
    }
}

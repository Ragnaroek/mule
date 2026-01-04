use eframe::egui;
use egui::{ColorImage, TextureHandle, load::SizedTexture};

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
                if ui.button("Upload").clicked() {}
            });
        });
    }
}

fn main() {
    eprintln!("native app TODO")
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("mule_canvas")
            .expect("Failed to find mule_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("mule_canvas was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(MuleApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

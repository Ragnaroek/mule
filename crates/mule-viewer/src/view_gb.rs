use egui::{Color32, Frame, Margin, Stroke};
use mule_gb::{self, GBBinary};

const SIDE_SEG_MARGIN: i8 = 8;

pub fn show_view_gb(ctx: &egui::Context, gb_binary: &GBBinary) {
    egui::SidePanel::left("master_panel")
        .resizable(true)
        .default_width(300.0)
        .frame(Frame::new().inner_margin(Margin::same(SIDE_SEG_MARGIN)))
        .show(ctx, |ui| {
            tile_frame(ui, "Restart Calls", |ui| {
                ui.label("Non-default restarts: 4");
            });
            ui.add_space(SIDE_SEG_MARGIN as f32);

            tile_frame(ui, "Interrupts", |ui| {
                ui.label("Non-default interrupts: 4");
            });
            ui.add_space(SIDE_SEG_MARGIN as f32);

            tile_frame(ui, "Header", |ui| {
                ui.label("Non-default interrupts: 4");
            });
            ui.add_space(SIDE_SEG_MARGIN as f32);

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

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Details");
        ui.separator();
    });
}

fn tile_frame(ui: &mut egui::Ui, title: &str, body: impl FnOnce(&mut egui::Ui)) {
    Frame::new()
        //.fill(Color32::from_rgb(52, 152, 219))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgb(230, 126, 34),
            //ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .inner_margin(Margin::same(6))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(title)
                    .monospace()
                    .strong()
                    .color(Color32::from_rgb(230, 126, 34)), //.color(egui::Color32::LIGHT_YELLOW),
            );
            ui.separator();
            body(ui);
        });
}

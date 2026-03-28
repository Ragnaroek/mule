/// common functions and types for the various binary view implementations
use egui::{Color32, Frame, Margin, Response, Stroke};

pub trait BinaryViewWidget {
    fn show(&mut self, ui: &mut egui::Ui);
}

pub struct TileWidget {
    title: String,
    selected: bool,
    selectable: bool,
}

impl TileWidget {
    pub fn new(title: String) -> TileWidget {
        return TileWidget {
            title,
            selected: false,
            selectable: true,
        };
    }

    /// disable click selection of the whole tile.
    /// Necessary if the inner components of the tile want
    /// to receive clicks and selection is handled there.
    pub fn not_selectable(&mut self) {
        self.selectable = false;
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn show(&self, ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) -> Response {
        let selected_colour = Color32::from_rgb(230, 126, 34);

        let inner_response = Frame::new()
            .stroke(Stroke::new(
                1.0,
                if self.selected {
                    selected_colour
                } else {
                    ui.visuals().widgets.noninteractive.bg_stroke.color
                },
            ))
            .inner_margin(Margin::same(6))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(&self.title)
                        .monospace()
                        .strong()
                        .color(Color32::from_rgb(230, 126, 34)), //.color(egui::Color32::LIGHT_YELLOW),
                );
                ui.separator();
                body(ui);
            });

        if self.selectable {
            ui.interact(
                inner_response.response.rect, // The area of the frame
                ui.id().with(&self.title),    // A unique ID for this interaction
                egui::Sense::click(),         // We want to sense clicks
            )
        } else {
            inner_response.response
        }
    }
}

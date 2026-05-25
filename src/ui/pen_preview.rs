use eframe::egui::Ui;

pub struct PenPreview;

impl PenPreview {
    pub fn ui(ui: &mut Ui, show_pen_preview: &mut bool) {
        ui.horizontal(|ui| {
            ui.label("Pen Preview:");
            ui.toggle_value(show_pen_preview, "Show")
                .on_hover_text("Toggle pen width preview — shows paths at their physical stroke width");
            if *show_pen_preview {
                let _ = ui.label("(paths render at physical pen width)");
            }
        });
    }
}

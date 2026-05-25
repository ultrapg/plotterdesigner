use eframe::egui::Ui;

#[derive(Clone, Debug, PartialEq)]
pub enum Tool {
    Select,
    Pan,
}

impl Default for Tool {
    fn default() -> Self {
        Self::Select
    }
}

pub struct Toolbar;

impl Toolbar {
    pub fn ui(ui: &mut Ui, tool: &mut Tool) {
        ui.horizontal(|ui| {
            ui.label("Tool:");
            ui.selectable_value(tool, Tool::Select, "Select")
                .on_hover_text("Select and edit paths (left-click to select, drag to move/resize/rotate)");
            ui.selectable_value(tool, Tool::Pan, "Pan")
                .on_hover_text("Pan the canvas (or hold Shift + drag, or right-click + drag)");
        });
    }
}

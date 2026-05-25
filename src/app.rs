use std::fs;

use eframe::egui::{self, DragValue};
use eframe::Frame as EFrame;

use crate::canvas::{InteractionState, InteractiveCanvas, ViewportTransform};
use crate::core::{Document, Unit};
use crate::export::ExportOptions;
use crate::generators::GeneratorParams;
use crate::import::SvgImporter;
use crate::manipulate::{self, ManipulateState};
use crate::ui::dialogs::{DialogAction, Dialogs};
use crate::ui::generator_panel::GeneratorPanel;
use crate::ui::layer_panel::LayerPanel;
use crate::ui::pen_preview::PenPreview;
use crate::ui::toolbar::{Tool, Toolbar};

pub struct App {
    pub document: Document,
    pub viewport: ViewportTransform,
    pub generator_params: GeneratorParams,
    #[allow(dead_code)]
    pub manipulate_state: ManipulateState,
    pub tool: Tool,
    pub interaction: InteractionState,
    pub selected_path_ids: Vec<uuid::Uuid>,
    pub show_pen_preview: bool,
    pub show_canvas_settings: bool,
    pub show_generator_panel: bool,
    pub show_layer_panel: bool,
    pub export_options: ExportOptions,
    pub dialog_action: DialogAction,
    pub show_export_dialog: bool,
    pub show_import_dialog: bool,
    pub show_save_dialog: bool,
    pub show_open_dialog: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            document: Document::default(),
            viewport: ViewportTransform::default(),
            generator_params: GeneratorParams::default(),
            manipulate_state: ManipulateState::default(),
            tool: Tool::Select,
            interaction: InteractionState::default(),
            selected_path_ids: Vec::new(),
            show_pen_preview: false,
            show_canvas_settings: false,
            show_generator_panel: true,
            show_layer_panel: true,
            export_options: ExportOptions::default(),
            dialog_action: DialogAction::None,
            show_export_dialog: false,
            show_import_dialog: false,
            show_save_dialog: false,
            show_open_dialog: false,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut EFrame) {
        // --- Menu bar ---
        egui::Panel::top("menubar").show_inside(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Export SVG").clicked() {
                        self.show_export_dialog = true;
                        ui.close();
                    }
                    if ui.button("Save Project").clicked() {
                        self.show_save_dialog = true;
                        ui.close();
                    }
                    if ui.button("Open Project").clicked() {
                        self.show_open_dialog = true;
                        ui.close();
                    }
                    ui.separator();
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                })
                .response
                .on_hover_text("Export SVG, save/open .pdp project files");

                ui.menu_button("Edit", |ui| {
                    let selected_ids = self.selected_path_ids.clone();

                    if ui.button("Duplicate").clicked() {
                        manipulate::duplicate_selection(&mut self.document, &selected_ids);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Clear All").clicked() {
                        if let Some(layer) = self.document.active_layer_mut() {
                            layer.paths.clear();
                        }
                        self.selected_path_ids.clear();
                        ui.close();
                    }
                });
                // Tooltip is on the menu_button itself via the whole bar

                ui.menu_button("Import", |ui| {
                    if ui.button("Import SVG...").clicked() {
                        self.show_import_dialog = true;
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_generator_panel, "Generator Panel")
                        .on_hover_text("Show/hide the generator panel on the right");
                    ui.checkbox(&mut self.show_layer_panel, "Layer Panel")
                        .on_hover_text("Show/hide the layer panel on the left");
                    ui.separator();
                    ui.checkbox(&mut self.show_pen_preview, "Pen Preview")
                        .on_hover_text("Preview stroke widths as they would appear on paper");
                });

                ui.menu_button("Canvas", |ui| {
                    if ui.button("Canvas Settings...").clicked() {
                        self.show_canvas_settings = !self.show_canvas_settings;
                        ui.close();
                    }
                });
            });
        });

        // --- Toolbar ---
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            Toolbar::ui(ui, &mut self.tool);
        });

        // --- Canvas Settings Window ---
        if self.show_canvas_settings {
            egui::Window::new("Canvas Settings")
                .default_size(egui::vec2(250.0, 250.0))
                .open(&mut self.show_canvas_settings)
                .show(ui, |ui| {
                    let doc = &mut self.document;
                    let factor = doc.unit.factor();
                    let label = doc.unit.label();
                    let mut display_w = doc.paper_width_cm * factor;
                    let mut display_h = doc.paper_height_cm * factor;

                    ui.horizontal(|ui| {
                        ui.label("Width:");
                        ui.add(DragValue::new(&mut display_w).speed(0.1).range(1.0..=200.0 * factor))
                            .on_hover_text("Paper width in the selected unit");
                        ui.label(label);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Height:");
                        ui.add(DragValue::new(&mut display_h).speed(0.1).range(1.0..=200.0 * factor))
                            .on_hover_text("Paper height in the selected unit");
                        ui.label(label);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Unit:");
                        ui.selectable_value(&mut doc.unit, Unit::Cm, "cm")
                            .on_hover_text("Centimeters");
                        ui.selectable_value(&mut doc.unit, Unit::Mm, "mm")
                            .on_hover_text("Millimeters");
                    });

                    ui.separator();

                    ui.checkbox(&mut doc.snap_to_grid, "Snap to grid")
                        .on_hover_text("When enabled, path coordinates snap to grid increments");
                    if doc.snap_to_grid {
                        ui.add(DragValue::new(&mut doc.grid_size).speed(0.5).range(0.1..=100.0).prefix("Grid: "))
                            .on_hover_text("Distance between grid lines");
                    }

                    ui.separator();

                    ui.checkbox(&mut doc.show_border, "Show border")
                        .on_hover_text("Display the paper boundary on canvas");
                    if doc.show_border {
                        ui.checkbox(&mut doc.border_dashed, "Dashed")
                            .on_hover_text("Toggle between dashed and solid border line");
                    }
                });
        }

        // --- Side panel: Layers ---
        if self.show_layer_panel {
            egui::Panel::left("layer_panel")
                .default_size(200.0)
                .resizable(true)
                .show_inside(ui, |ui| {
                    LayerPanel::ui(
                        ui,
                        &mut self.document,
                        &mut self.selected_path_ids,
                    );
                });
        }

        // --- Side panel: Generator ---
        if self.show_generator_panel {
            egui::Panel::right("generator_panel")
                .default_size(280.0)
                .resizable(true)
                .show_inside(ui, |ui| {
                    GeneratorPanel::ui(ui, &mut self.document, &mut self.generator_params);
                });
        }

        // --- Central canvas ---
        egui::CentralPanel::default().show_inside(ui, |ui| {
            InteractiveCanvas::ui(
                ui,
                &self.document,
                &mut self.viewport,
                &self.tool,
                &mut self.selected_path_ids,
                &mut self.interaction,
                self.show_pen_preview,
            );
        });

        // --- Bottom status bar ---
        egui::Panel::bottom("status").show_inside(ui, |ui| {
            PenPreview::ui(ui, &mut self.show_pen_preview);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Zoom: {:.1}%", self.viewport.zoom * 100.0));
                ui.label(format!(
                    "Paths: {}",
                    self.document
                        .layers
                        .iter()
                        .map(|l| l.paths.len())
                        .sum::<usize>()
                ));
            });
        });

        // --- Dialogs ---
        let dlg_ctx = ui.ctx().clone();
        if self.show_export_dialog {
            Dialogs::export_svg_modal(&dlg_ctx, &self.document, &mut self.dialog_action, &mut self.export_options);
        }
        if self.show_import_dialog {
            Dialogs::import_svg_modal(&dlg_ctx, &mut self.dialog_action);
        }
        if self.show_save_dialog {
            Dialogs::save_project_modal(&dlg_ctx, &mut self.dialog_action);
        }
        if self.show_open_dialog {
            Dialogs::open_project_modal(&dlg_ctx, &mut self.dialog_action);
        }

        // Handle dialog actions
        match std::mem::replace(&mut self.dialog_action, DialogAction::None) {
            DialogAction::ExportSvg => {
                self.show_export_dialog = false;
                self.handle_export();
            }
            DialogAction::ImportSvg => {
                self.show_import_dialog = false;
                self.handle_import();
            }
            DialogAction::SaveProject => {
                self.show_save_dialog = false;
                self.handle_save();
            }
            DialogAction::OpenProject => {
                self.show_open_dialog = false;
                self.handle_open();
            }
            DialogAction::Dismiss => {
                self.show_export_dialog = false;
                self.show_import_dialog = false;
                self.show_save_dialog = false;
                self.show_open_dialog = false;
            }
            DialogAction::None => {}
        }

        // Apply pending interaction actions
        if let Some(action) = self.interaction.pending_action.take() {
            use crate::canvas::PendingAction;
            match action {
                PendingAction::TranslatePaths { ids, dx, dy } => {
                    for id in &ids {
                        manipulate::translate_path(&mut self.document, *id, dx, dy);
                    }
                }
                PendingAction::ScalePaths { ids, cx, cy, sx, sy } => {
                    for id in &ids {
                        manipulate::scale_path(&mut self.document, *id, cx, cy, sx, sy);
                    }
                }
                PendingAction::EditPoint { id, element_idx, new_x, new_y } => {
                    manipulate::edit_point(&mut self.document, id, element_idx, new_x, new_y);
                }
                PendingAction::RotatePaths { ids, cx, cy, angle_rad } => {
                    manipulate::rotate_paths(&mut self.document, &ids, cx, cy, angle_rad);
                }
            }
        }

        // ── Keyboard shortcuts ──
        let selected = self.selected_path_ids.clone();
        let (del, left, right, up, down, ctrl_d, shift) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::ArrowRight),
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.modifiers.ctrl && i.key_pressed(egui::Key::D),
                i.modifiers.shift,
            )
        });
        if del && !selected.is_empty() {
            for layer in &mut self.document.layers {
                layer.paths.retain(|p| !selected.contains(&p.id));
            }
            self.selected_path_ids.clear();
        }
        if !selected.is_empty() {
            let nudge = if shift { 10.0 } else { 1.0 };
            if left {
                for id in &selected { manipulate::translate_path(&mut self.document, *id, -nudge, 0.0); }
            }
            if right {
                for id in &selected { manipulate::translate_path(&mut self.document, *id, nudge, 0.0); }
            }
            if up {
                for id in &selected { manipulate::translate_path(&mut self.document, *id, 0.0, -nudge); }
            }
            if down {
                for id in &selected { manipulate::translate_path(&mut self.document, *id, 0.0, nudge); }
            }
            if ctrl_d {
                manipulate::duplicate_selection(&mut self.document, &selected);
            }
        }

        // Request continuous repaint for smooth interactions
        ui.ctx().request_repaint();
    }
}

impl App {
    fn handle_export(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("SVG", &["svg"])
                .set_file_name("export.svg")
                .save_file()
            {
                let svg = crate::export::SvgWriter::write_with_options(&self.document, &self.export_options);
                if let Err(e) = fs::write(&path, &svg) {
                    eprintln!("Failed to write SVG: {}", e);
                }
            }
        }
    }

    fn handle_import(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("SVG", &["svg"])
                .pick_file()
            {
                match fs::read_to_string(&path) {
                    Ok(content) => match SvgImporter::import_from_str(&content) {
                        Ok(doc) => {
                            self.document = doc;
                            self.viewport = ViewportTransform::default();
                            self.selected_path_ids.clear();
                        }
                        Err(e) => {
                            eprintln!("Import error: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read file: {}", e);
                    }
                }
            }
        }
    }

    fn handle_save(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PlotterDesigner", &["pdp"])
                .set_file_name("project.pdp")
                .save_file()
            {
                match ron::ser::to_string(&self.document) {
                    Ok(data) => {
                        if let Err(e) = fs::write(&path, data) {
                            eprintln!("Failed to save project: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Serialization error: {}", e);
                    }
                }
            }
        }
    }

    fn handle_open(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PlotterDesigner", &["pdp"])
                .pick_file()
            {
                match fs::read_to_string(&path) {
                    Ok(content) => match ron::de::from_str::<Document>(&content) {
                        Ok(doc) => {
                            self.document = doc;
                            self.viewport = ViewportTransform::default();
                            self.selected_path_ids.clear();
                        }
                        Err(e) => {
                            eprintln!("Deserialization error: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read file: {}", e);
                    }
                }
            }
        }
    }
}

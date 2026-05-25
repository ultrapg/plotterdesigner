use eframe::egui;

use crate::core::Document;
use crate::export::{ExportOptions, SvgWriter};

pub enum DialogAction {
    None,
    ExportSvg,
    ImportSvg,
    SaveProject,
    OpenProject,
    Dismiss,
}

pub struct Dialogs;

impl Dialogs {
    pub fn export_svg_modal(ctx: &egui::Context, document: &Document, action: &mut DialogAction, options: &mut ExportOptions) {
        let svg_content = SvgWriter::write_with_options(document, options);

        egui::Window::new("Export SVG")
            .id(egui::Id::new("export_svg_dialog"))
            .resizable(true)
            .default_size([600.0, 500.0])
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label("Options:");
                    ui.checkbox(&mut options.remove_overlaps, "Remove overlapping paths (EXPERIMENTAL)")
                        .on_hover_text("When enabled, detects and removes line segments that are nearly coincident. This reduces redundant pen strokes but may alter the output. Use with caution.");
                    if options.remove_overlaps {
                        ui.label("⚠ This feature is experimental. It may remove paths it shouldn't.")
                            .on_hover_text("Overlap detection is basic: it only removes segments with nearly identical endpoints. Complex intersections are not handled.");
                    }

                    ui.separator();
                    ui.label("SVG Preview (first 500 chars):");
                    ui.separator();
                    let preview = if svg_content.len() > 500 {
                        format!("{}...", &svg_content[..500])
                    } else {
                        svg_content.clone()
                    };
                    ui.monospace(preview);
                    ui.separator();
                    if ui.button("Save to file...").on_hover_text("Export SVG to a file on disk").clicked() {
                        *action = DialogAction::ExportSvg;
                    }
                    if ui.button("Close").on_hover_text("Close this dialog without exporting").clicked() {
                        *action = DialogAction::Dismiss;
                    }
                });
            });
    }

    pub fn import_svg_modal(ctx: &egui::Context, action: &mut DialogAction) {
        egui::Window::new("Import SVG")
            .id(egui::Id::new("import_svg_dialog"))
            .resizable(true)
            .default_size([400.0, 150.0])
            .show(ctx, |ui| {
                ui.label("Select an SVG file to import.");
                if ui.button("Open file...").on_hover_text("Pick an SVG file from disk").clicked() {
                    *action = DialogAction::ImportSvg;
                }
                if ui.button("Cancel").on_hover_text("Cancel import").clicked() {
                    *action = DialogAction::Dismiss;
                }
            });
    }

    pub fn save_project_modal(ctx: &egui::Context, action: &mut DialogAction) {
        egui::Window::new("Save Project")
            .id(egui::Id::new("save_project_dialog"))
            .default_size([400.0, 100.0])
            .show(ctx, |ui| {
                ui.label("Save project as .pdp file.");
                if ui.button("Save...").on_hover_text("Save project to a .pdp file").clicked() {
                    *action = DialogAction::SaveProject;
                }
                if ui.button("Cancel").on_hover_text("Cancel save").clicked() {
                    *action = DialogAction::Dismiss;
                }
            });
    }

    pub fn open_project_modal(ctx: &egui::Context, action: &mut DialogAction) {
        egui::Window::new("Open Project")
            .id(egui::Id::new("open_project_dialog"))
            .default_size([400.0, 100.0])
            .show(ctx, |ui| {
                ui.label("Open a .pdp project file.");
                if ui.button("Open...").on_hover_text("Open a .pdp project file").clicked() {
                    *action = DialogAction::OpenProject;
                }
                if ui.button("Cancel").on_hover_text("Cancel open").clicked() {
                    *action = DialogAction::Dismiss;
                }
            });
    }
}

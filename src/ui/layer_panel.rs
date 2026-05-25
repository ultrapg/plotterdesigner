use eframe::egui::{Color32, DragValue, Ui};
use kurbo::{BezPath, PathEl, Shape as KurboShape};
use uuid::Uuid;

use crate::canvas::compute_collective_bbox;
use crate::core::Document;
use crate::manipulate;

fn approximate_path_length(path: &BezPath) -> f64 {
    let mut len = 0.0;
    let mut prev: Option<kurbo::Point> = None;
    for el in path.elements() {
        let pt = match el {
            PathEl::MoveTo(p) => *p,
            PathEl::LineTo(p) => *p,
            PathEl::QuadTo(_, p) => *p,
            PathEl::CurveTo(_, _, p) => *p,
            PathEl::ClosePath => continue,
        };
        if let Some(a) = prev {
            len += ((pt.x - a.x).powi(2) + (pt.y - a.y).powi(2)).sqrt();
        }
        prev = Some(pt);
    }
    len
}

fn segment_count(path: &BezPath) -> usize {
    path.elements().iter().filter(|el| matches!(el, PathEl::LineTo(_) | PathEl::QuadTo(_, _) | PathEl::CurveTo(_, _, _))).count()
}

pub struct LayerPanel;

impl LayerPanel {
    pub fn ui(
        ui: &mut Ui,
        document: &mut Document,
        selected_path_ids: &mut Vec<Uuid>,
    ) {
        ui.heading("Layers");
        ui.separator();

        let mut layer_to_remove: Option<usize> = None;

        let num_layers = document.layers.len();
        for idx in 0..num_layers {
            let layer = &document.layers[idx];
            let mut new_visible = layer.visible;
            let mut new_name = layer.name.clone();

                let delete_clicked = ui.horizontal(|ui| {
                ui.radio_value(&mut document.active_layer_idx, idx, "");

                let vis_label = if new_visible { "V" } else { "H" };
                if ui.toggle_value(&mut new_visible, vis_label)
                    .on_hover_text("Toggle layer visibility")
                    .changed()
                {
                    document.layers[idx].visible = new_visible;
                }
                if ui.text_edit_singleline(&mut new_name)
                    .on_hover_text("Rename layer")
                    .lost_focus()
                {
                    document.layers[idx].name.clone_from(&new_name);
                }
                if idx > 0 && ui.button("^").on_hover_text("Move layer up").clicked() {
                    document.layers.swap(idx, idx - 1);
                    if document.active_layer_idx == idx {
                        document.active_layer_idx = idx - 1;
                    } else if document.active_layer_idx == idx - 1 {
                        document.active_layer_idx = idx;
                    }
                }
                if idx + 1 < num_layers && ui.button("v").on_hover_text("Move layer down").clicked() {
                    document.layers.swap(idx, idx + 1);
                    if document.active_layer_idx == idx {
                        document.active_layer_idx = idx + 1;
                    } else if document.active_layer_idx == idx + 1 {
                        document.active_layer_idx = idx;
                    }
                }
                ui.button("X").on_hover_text("Delete layer").clicked()
            }).inner;

            if delete_clicked {
                layer_to_remove = Some(idx);
                continue;
            }

            // Path list (using unique ID for each layer)
            let path_count = document.layers[idx].paths.len();
            let ply_label = format!("Paths ({})", path_count);
            let collapsing_id = ui.id().with(idx);
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), collapsing_id, true)
                .show_header(ui, |ui: &mut Ui| { ui.label(&ply_label); })
                .body(|ui: &mut Ui| {
                    for pidx in 0..document.layers[idx].paths.len() {
                        let path_id = document.layers[idx].paths[pidx].id;
                        let is_sel = selected_path_ids.contains(&path_id);

                        // Each path row is a collapsible section
                        let path_collapsing_id = ui.id().with(format!("path_{}_{}", idx, pidx));
                        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), path_collapsing_id, false)
                            .show_header(ui, |ui: &mut Ui| {
                                ui.horizontal(|ui| {
                                    // Editable name
                                    let mut pname = if document.layers[idx].paths[pidx].name.is_empty() {
                                        format!("Path {}", pidx + 1)
                                    } else {
                                        document.layers[idx].paths[pidx].name.clone()
                                    };
                                    ui.set_min_width(100.0);
                                    if ui.text_edit_singleline(&mut pname)
                                        .on_hover_text("Rename path")
                                        .lost_focus()
                                    {
                                        document.layers[idx].paths[pidx].name = pname;
                                    }

                                    // Select toggle
                                    let sel_label = if is_sel { "[x]" } else { "[ ]" };
                                    if ui.selectable_label(is_sel, sel_label)
                                        .on_hover_text("Click to select (Shift for multi-select)")
                                        .clicked()
                                    {
                                        let shift = ui.input(|i| i.modifiers.shift);
                                        if shift {
                                            if let Some(pos) = selected_path_ids.iter().position(|&id| id == path_id) {
                                                selected_path_ids.remove(pos);
                                            } else {
                                                selected_path_ids.push(path_id);
                                            }
                                        } else {
                                            *selected_path_ids = vec![path_id];
                                        }
                                    }

                                    // Fill toggle
                                    let mut filled = document.layers[idx].paths[pidx].filled;
                                    let fill_label = if filled { "Fill" } else { "Out" };
                                    if ui.toggle_value(&mut filled, fill_label)
                                        .on_hover_text("Toggle filled/outline rendering")
                                        .changed()
                                    {
                                        document.layers[idx].paths[pidx].filled = filled;
                                    }
                                });
                            })
                            .body(|ui: &mut Ui| {
                                let path = &document.layers[idx].paths[pidx];
                                let bbox = path.geometry.bounding_box();
                                let unit = &document.unit;
                                let factor = unit.factor();
                                let label = unit.label();
                                let inv_factor = 1.0 / crate::core::CM_TO_UNITS;

                                let display_x = bbox.x0 * inv_factor * factor;
                                let display_y = bbox.y0 * inv_factor * factor;
                                let display_w = bbox.width() * inv_factor * factor;
                                let display_h = bbox.height() * inv_factor * factor;

                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    ui.label(format!("Segments: {}", segment_count(&path.geometry)));
                                    let plen = approximate_path_length(&path.geometry);
                                    let display_len = plen * inv_factor * factor;
                                    ui.label(format!("Length: {:.1} {}", display_len, label));
                                });

                                ui.horizontal(|ui| {
                                    let mut d_w = display_w;
                                    if ui.add(DragValue::new(&mut d_w).speed(0.05).prefix("W: "))
                                        .on_hover_text("Path width")
                                        .changed() {
                                        let new_w = d_w / factor * crate::core::CM_TO_UNITS;
                                        let old_w = if bbox.width() > 0.0 { bbox.width() } else { 1.0 };
                                        if (new_w - old_w).abs() > 0.01 {
                                            let sx = new_w / old_w;
                                            let cx = (bbox.x0 + bbox.x1) / 2.0;
                                            let cy = (bbox.y0 + bbox.y1) / 2.0;
                                            manipulate::scale_path(document, path_id, cx, cy, sx, 1.0);
                                        }
                                    }
                                    ui.label(label);
                                });
                                ui.horizontal(|ui| {
                                    let mut d_h = display_h;
                                    if ui.add(DragValue::new(&mut d_h).speed(0.05).prefix("H: "))
                                        .on_hover_text("Path height")
                                        .changed() {
                                        let new_h = d_h / factor * crate::core::CM_TO_UNITS;
                                        let old_h = if bbox.height() > 0.0 { bbox.height() } else { 1.0 };
                                        if (new_h - old_h).abs() > 0.01 {
                                            let sy = new_h / old_h;
                                            let cx = (bbox.x0 + bbox.x1) / 2.0;
                                            let cy = (bbox.y0 + bbox.y1) / 2.0;
                                            manipulate::scale_path(document, path_id, cx, cy, 1.0, sy);
                                        }
                                    }
                                    ui.label(label);
                                });
                                ui.horizontal(|ui| {
                                    let mut d_x = display_x;
                                    if ui.add(DragValue::new(&mut d_x).speed(0.05).prefix("X: ")).changed() {
                                        let new_x = d_x / factor * crate::core::CM_TO_UNITS;
                                        let dx = new_x - bbox.x0;
                                        if dx.abs() > 0.01 {
                                            manipulate::translate_path(document, path_id, dx, 0.0);
                                        }
                                    }
                                    ui.label(label);
                                });
                                ui.horizontal(|ui| {
                                    let mut d_y = display_y;
                                    if ui.add(DragValue::new(&mut d_y).speed(0.05).prefix("Y: ")).changed() {
                                        let new_y = d_y / factor * crate::core::CM_TO_UNITS;
                                        let dy = new_y - bbox.y0;
                                        if dy.abs() > 0.01 {
                                            manipulate::translate_path(document, path_id, 0.0, dy);
                                        }
                                    }
                                    ui.label(label);
                                });
                            });
                    }
                });
        }

        if let Some(idx) = layer_to_remove {
            document.remove_layer(idx);
            selected_path_ids.clear();
        }

        ui.separator();

        if ui.button("+ Add Layer").on_hover_text("Create a new layer").clicked() {
            let num = document.layers.len() + 1;
            document.add_layer(crate::core::Layer::new(
                format!("Layer {}", num),
                crate::core::Pen {
                    color: Color32::from_rgb(
                        rand::random::<u8>(),
                        rand::random::<u8>(),
                        rand::random::<u8>(),
                    ),
                    ..Default::default()
                },
            ));
        }

        // Selection transform controls
        if !selected_path_ids.is_empty() {
            ui.separator();
            ui.heading("Selected");

            if selected_path_ids.len() > 1 {
                ui.label(format!("{} items selected", selected_path_ids.len()));
            }

            // Editable name for single selection
            if selected_path_ids.len() == 1 {
                let sid = selected_path_ids[0];
                let mut found_name = String::new();
                for layer in &document.layers {
                    for path in &layer.paths {
                        if path.id == sid {
                            found_name = if path.name.is_empty() {
                                "Path".into()
                            } else {
                                path.name.clone()
                            };
                            break;
                        }
                    }
                }
                let mut name_copy = found_name.clone();
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    if ui.text_edit_singleline(&mut name_copy).lost_focus() {
                        for layer in &mut document.layers {
                            for path in &mut layer.paths {
                                if path.id == sid {
                                    path.name.clone_from(&name_copy);
                                    break;
                                }
                            }
                        }
                    }
                });
            }

            // Fill toggle
            let mut all_filled = true;
            let mut any_filled = false;
            for layer in &document.layers {
                for path in &layer.paths {
                    if selected_path_ids.contains(&path.id) {
                        if path.filled { any_filled = true; } else { all_filled = false; }
                    }
                }
            }
            let mut fill_state = all_filled;
            if ui.checkbox(&mut fill_state, "Filled")
                .on_hover_text("Toggle fill for all selected paths")
                .changed() {
                for layer in &mut document.layers {
                    for path in &mut layer.paths {
                        if selected_path_ids.contains(&path.id) {
                            path.filled = fill_state;
                        }
                    }
                }
            }
            if !all_filled && any_filled {
                // Mixed state indicator — reset on next click
            }

            if let Some(bbox) = compute_collective_bbox(document, selected_path_ids) {
                let unit = &document.unit;
                let factor = unit.factor();
                let label = unit.label();
                let inv_factor = 1.0 / crate::core::CM_TO_UNITS;
                let display_x = bbox.x0 * inv_factor * factor;
                let display_y = bbox.y0 * inv_factor * factor;
                let mut display_w = bbox.width() * inv_factor * factor;
                let mut display_h = bbox.height() * inv_factor * factor;

                ui.horizontal(|ui| {
                    ui.label("W:");
                    if ui.add(DragValue::new(&mut display_w).speed(0.05).range(0.01..=10000.0 * factor)).changed() {
                        let new_w = display_w / factor * crate::core::CM_TO_UNITS;
                        let old_w = if bbox.width() > 0.0 { bbox.width() } else { 1.0 };
                        let sx = new_w / old_w;
                        let bbox_center_x = (bbox.x0 + bbox.x1) / 2.0;
                        let bbox_center_y = (bbox.y0 + bbox.y1) / 2.0;
                        for id in selected_path_ids.iter() {
                            manipulate::scale_path(document, *id, bbox_center_x, bbox_center_y, sx, 1.0);
                        }
                    }
                    ui.label(label);
                });
                ui.horizontal(|ui| {
                    ui.label("H:");
                    if ui.add(DragValue::new(&mut display_h).speed(0.05).range(0.01..=10000.0 * factor)).changed() {
                        let new_h = display_h / factor * crate::core::CM_TO_UNITS;
                        let old_h = if bbox.height() > 0.0 { bbox.height() } else { 1.0 };
                        let sy = new_h / old_h;
                        let bbox_center_x = (bbox.x0 + bbox.x1) / 2.0;
                        let bbox_center_y = (bbox.y0 + bbox.y1) / 2.0;
                        for id in selected_path_ids.iter() {
                            manipulate::scale_path(document, *id, bbox_center_x, bbox_center_y, 1.0, sy);
                        }
                    }
                    ui.label(label);
                });

                ui.horizontal(|ui| {
                    ui.label("X:");
                    let mut display_x_val = display_x;
                    if ui.add(DragValue::new(&mut display_x_val).speed(0.05)).changed() {
                        let new_x = display_x_val / factor * crate::core::CM_TO_UNITS;
                        let old_x = bbox.x0;
                        let dx = new_x - old_x;
                        for id in selected_path_ids.iter() {
                            manipulate::translate_path(document, *id, dx, 0.0);
                        }
                    }
                    ui.label(label);
                });
                ui.horizontal(|ui| {
                    ui.label("Y:");
                    let mut display_y_val = display_y;
                    if ui.add(DragValue::new(&mut display_y_val).speed(0.05)).changed() {
                        let new_y = display_y_val / factor * crate::core::CM_TO_UNITS;
                        let old_y = bbox.y0;
                        let dy = new_y - old_y;
                        for id in selected_path_ids.iter() {
                            manipulate::translate_path(document, *id, 0.0, dy);
                        }
                    }
                    ui.label(label);
                });
            }

            if ui.button("Duplicate").on_hover_text("Clone selected paths").clicked() {
                let selected = selected_path_ids.clone();
                manipulate::duplicate_selection(document, &selected);
            }
            if ui.button("Delete").on_hover_text("Remove selected paths").clicked() {
                for layer in &mut document.layers {
                    layer.paths.retain(|p| !selected_path_ids.contains(&p.id));
                }
                selected_path_ids.clear();
            }
        }
    }
}

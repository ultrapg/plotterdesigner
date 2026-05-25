use eframe::egui::{self, Color32, Pos2, Rect, Shape, Stroke, StrokeKind};
use kurbo::{Affine, BezPath, PathEl, Shape as KurboShape};
use uuid::Uuid;

use crate::canvas::interaction::*;
use crate::canvas::ViewportTransform;
use crate::core::{Document, CM_TO_UNITS};
use crate::ui::toolbar::Tool;

const HANDLE_SIZE: f32 = 10.0;
const HANDLE_HIT_RADIUS: f32 = 14.0;
const ROTATE_HANDLE_RADIUS: f32 = 8.0;
const VERTEX_HIT_RADIUS_PX: f64 = 14.0;
const SELECT_TOLERANCE_PX: f64 = 12.0;
const MOVE_TOLERANCE_PX: f64 = 16.0;

/// Minimum bbox size (world units) so degenerate paths (lines) get visible handles.
const MIN_BBOX_SIZE: f64 = 2.0;

pub struct InteractiveCanvas;

impl InteractiveCanvas {
    #[allow(clippy::too_many_arguments)]
    pub fn ui(
        ui: &mut egui::Ui,
        document: &Document,
        viewport: &mut ViewportTransform,
        tool: &Tool,
        selected_path_ids: &mut Vec<Uuid>,
        interaction: &mut InteractionState,
        show_pen_preview: bool,
    ) {
        let (response, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );

        let rect = response.rect;

        // ── Pan: middle mouse OR shift+drag OR right mouse ──
        let is_pan = response.dragged_by(egui::PointerButton::Middle)
            || response.dragged_by(egui::PointerButton::Secondary)
            || (response.dragged() && ui.input(|i| i.modifiers.shift))
            || (response.dragged() && *tool == Tool::Pan);
        if is_pan {
            let delta = response.drag_delta();
            viewport.pan_by(delta.x as f64, delta.y as f64);
        }

        // ── Interaction (Select tool only, not when panning) ──
        if *tool == Tool::Select && !is_pan {
            Self::handle_interaction(
                &response,
                ui,
                rect,
                viewport,
                document,
                selected_path_ids,
                interaction,
            );
        }

        // ── Zoom ──
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta);
            if scroll.y != 0.0 {
                let center = rect.center();
                viewport.zoom_at(center.x as f64, center.y as f64, (1.0 + scroll.y * 0.002) as f64);
            }
        }

        // ── Background ──
        painter.rect_filled(rect, 0.0, Color32::from_gray(28));

        // ── Grid ──
        Self::draw_grid(&painter, rect, viewport);

        // ── Paper border ──
        if document.show_border {
            Self::draw_paper_border(&painter, rect, viewport, document);
        }

        // ── Paths ──
        let base_affine = viewport.affine();
        let visual_affine = interaction.visual_transform(viewport, rect);

        for layer in &document.layers {
            if !layer.visible {
                continue;
            }
            let base_width = if show_pen_preview {
                (layer.pen.width_mm * viewport.zoom * 3.7795) as f32
            } else {
                1.5
            };

            for path in &layer.paths {
                if !path.visible {
                    continue;
                }
                let is_sel = selected_path_ids.contains(&path.id);

                // ── Compute render geometry (point-editing override) ──
                let render_geom = match interaction.mode {
                    TransformMode::EditingPoint { path_id, element_idx } if path_id == path.id => {
                        let mut elements: Vec<PathEl> = path.geometry.elements().to_vec();
                        if element_idx < elements.len() {
                            let cur_w = viewport.screen_to_world(
                                (interaction.drag_current_screen.x - rect.left()) as f64,
                                (interaction.drag_current_screen.y - rect.top()) as f64,
                            );
                            elements[element_idx] = match elements[element_idx] {
                                PathEl::MoveTo(_) => PathEl::MoveTo(kurbo::Point::new(cur_w.x, cur_w.y)),
                                PathEl::LineTo(_) => PathEl::LineTo(kurbo::Point::new(cur_w.x, cur_w.y)),
                                PathEl::QuadTo(cp, _) => PathEl::QuadTo(cp, kurbo::Point::new(cur_w.x, cur_w.y)),
                                PathEl::CurveTo(cp1, cp2, _) => PathEl::CurveTo(cp1, cp2, kurbo::Point::new(cur_w.x, cur_w.y)),
                                PathEl::ClosePath => PathEl::ClosePath,
                            };
                            BezPath::from_vec(elements)
                        } else {
                            path.geometry.clone()
                        }
                    }
                    _ => path.geometry.clone(),
                };

                // Apply visual affine to selected paths during active transform
                let apply_visual = is_sel && matches!(interaction.mode, TransformMode::Moving | TransformMode::Scaling { .. } | TransformMode::Rotating);
                let path_affine = if apply_visual {
                    base_affine * visual_affine
                } else {
                    base_affine
                };

                let color = if is_sel {
                    Color32::YELLOW
                } else {
                    layer.pen.color
                };
                let width = if is_sel {
                    base_width.max(3.0)
                } else {
                    base_width
                };

                if path.filled {
                    let fill_shapes = bezpath_to_egui_fill_shapes(&render_geom, path_affine, rect, color);
                    for shape in fill_shapes {
                        painter.add(shape);
                    }
                }
                let shapes = bezpath_to_egui_shapes(&render_geom, path_affine, rect, color, width);
                for shape in shapes {
                    painter.add(shape);
                }

                // ── Vertex handles for point editing ──
                if is_sel && *tool == Tool::Select {
                    Self::draw_vertex_handles(&painter, rect, viewport, &render_geom, interaction);
                }
            }
        }

        // ── Bounding box + handles for selected paths ──
        if !selected_path_ids.is_empty() {
            Self::draw_selection_bbox(&painter, rect, viewport, document, selected_path_ids, interaction, visual_affine);
        }

        // ── Drag-select rectangle ──
        if interaction.mode == TransformMode::RectSelect {
            Self::draw_selection_rect(&painter, interaction);
        }
    }

    fn draw_vertex_handles(
        painter: &egui::Painter,
        screen_rect: Rect,
        viewport: &ViewportTransform,
        path: &BezPath,
        interaction: &InteractionState,
    ) {
        let affine = viewport.affine();
        for (idx, el) in path.elements().iter().enumerate() {
            let pt = match el {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => *p,
                PathEl::QuadTo(_, p) => *p,
                PathEl::CurveTo(_, _, p) => *p,
                PathEl::ClosePath => continue,
            };
            let t = affine * pt;
            let screen_pos = Pos2::new(screen_rect.left() + t.x as f32, screen_rect.top() + t.y as f32);

            let is_dragging = matches!(interaction.mode, TransformMode::EditingPoint { path_id: _, element_idx } if element_idx == idx);
            let color = if is_dragging {
                Color32::RED
            } else {
                Color32::WHITE
            };

            let r = Rect::from_center_size(screen_pos, egui::vec2(HANDLE_SIZE, HANDLE_SIZE));
            painter.rect_filled(r, 2.0, color);
            painter.rect_stroke(r, 2.0, Stroke::new(1.0, Color32::BLACK), StrokeKind::Middle);
        }
    }

    fn draw_selection_bbox(
        painter: &egui::Painter,
        screen_rect: Rect,
        viewport: &ViewportTransform,
        document: &Document,
        selected_path_ids: &[Uuid],
        _interaction: &InteractionState,
        visual_affine: Affine,
    ) {
        let mut raw_bbox: Option<kurbo::Rect> = None;
        for id in selected_path_ids {
            if let Some(path) = find_path_by_id(document, *id) {
                let r = path.geometry.bounding_box();
                raw_bbox = Some(raw_bbox.map_or(r, |a| a.union(r)));
            }
        }
        let bbox = match raw_bbox {
            Some(b) if b.width() > 0.0 && b.height() > 0.0 => b,
            Some(b) => {
                // Degenerate bbox: pad to minimum size so handles appear
                let cx = b.x0 + b.width() / 2.0;
                let cy = b.y0 + b.height() / 2.0;
                let w = b.width().max(MIN_BBOX_SIZE);
                let h = b.height().max(MIN_BBOX_SIZE);
                kurbo::Rect::from_center_size(kurbo::Point::new(cx, cy), (w, h))
            }
            None => return,
        };

        let handle_positions = compute_handle_screen_positions(
            bbox, viewport, screen_rect, visual_affine,
        );

        // Collect just the positions for rendering
        let resize_positions: Vec<Pos2> = handle_positions.iter()
            .filter(|(h, _)| *h != HandlePosition::Rotate)
            .map(|(_, p)| *p)
            .collect();
        let rotate_pos = handle_positions.iter()
            .find(|(h, _)| *h == HandlePosition::Rotate)
            .map(|(_, p)| *p);

        // Draw dashed bounding box (using corners)
        if !resize_positions.is_empty() {
            let c0 = resize_positions[0]; // TopLeft
            let c1 = resize_positions[2]; // TopRight
            let c2 = resize_positions[7]; // BottomRight
            let c3 = resize_positions[5]; // BottomLeft
            let pts = vec![c0, c1, c2, c3, c0];
            painter.add(Shape::dashed_line(
                &pts,
                Stroke::new(1.0, Color32::from_rgb(80, 120, 200)),
                5.0,
                3.0,
            ));
        }

        // Draw 8 resize handles
        let handle_color = Color32::from_rgb(140, 180, 255);
        for &pt in &resize_positions {
            let handle_rect = Rect::from_center_size(pt, egui::vec2(HANDLE_SIZE, HANDLE_SIZE));
            painter.rect_filled(handle_rect, 2.0, handle_color);
            painter.rect_stroke(handle_rect, 2.0, Stroke::new(1.0, Color32::WHITE), StrokeKind::Middle);

        }

        // Draw rotate handle (circle above top center)
        if let Some(rp) = rotate_pos {
            painter.circle_filled(rp, ROTATE_HANDLE_RADIUS, Color32::from_rgb(80, 200, 120));
            painter.circle_stroke(rp, ROTATE_HANDLE_RADIUS, Stroke::new(1.5, Color32::WHITE));
        }
    }

    fn draw_selection_rect(
        painter: &egui::Painter,
        interaction: &InteractionState,
    ) {
        let r = Rect::from_two_pos(interaction.drag_start_screen, interaction.drag_current_screen);
        let stroke = Stroke::new(1.0, Color32::from_rgb(100, 160, 255));
        painter.add(Shape::dashed_line(
            &[
                Pos2::new(r.left(), r.top()),
                Pos2::new(r.right(), r.top()),
                Pos2::new(r.right(), r.bottom()),
                Pos2::new(r.left(), r.bottom()),
                Pos2::new(r.left(), r.top()),
            ],
            stroke,
            4.0,
            4.0,
        ));
        painter.rect_filled(r, 0.0, Color32::from_rgba_premultiplied(100, 160, 255, 40));
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_interaction(
        response: &egui::Response,
        ui: &egui::Ui,
        screen_rect: Rect,
        viewport: &ViewportTransform,
        document: &Document,
        selected_path_ids: &mut Vec<Uuid>,
        interaction: &mut InteractionState,
    ) {
        // ── Drag end ──
        if response.drag_stopped() {
            match &interaction.mode {
                TransformMode::RectSelect => {
                    let screen_sel_rect = Rect::from_two_pos(
                        interaction.drag_start_screen,
                        interaction.drag_current_screen,
                    );
                    let world_p0 = viewport.screen_to_world(
                        (screen_sel_rect.left() - screen_rect.left()) as f64,
                        (screen_sel_rect.top() - screen_rect.top()) as f64,
                    );
                    let world_p1 = viewport.screen_to_world(
                        (screen_sel_rect.right() - screen_rect.left()) as f64,
                        (screen_sel_rect.bottom() - screen_rect.top()) as f64,
                    );
                    let world_rect = kurbo::Rect::from_points(world_p0, world_p1);
                    let shift = ui.input(|i| i.modifiers.shift);

                    let mut new_sel: Vec<Uuid> = if shift {
                        selected_path_ids.clone()
                    } else {
                        Vec::new()
                    };
                    for layer in &document.layers {
                        for path in &layer.paths {
                            if !path.visible { continue; }
                            if path_intersects_rect(&path.geometry, world_rect) {
                                if !new_sel.contains(&path.id) {
                                    new_sel.push(path.id);
                                }
                            }
                        }
                    }
                    *selected_path_ids = new_sel;
                }
                _ => {
                    if let Some(action) = interaction.compute_pending_action(viewport, screen_rect, selected_path_ids) {
                        interaction.pending_action = Some(action);
                    }
                }
            }
            interaction.mode = TransformMode::None;
            interaction.original_bbox = None;
            return;
        }

        // ── Drag start / click ──
        if response.drag_started() || response.clicked() {
            let click_pos = match response.interact_pointer_pos() {
                Some(p) => p,
                None => return,
            };

            interaction.drag_start_screen = click_pos;
            interaction.drag_current_screen = click_pos;

            let world_pos = viewport.screen_to_world(
                (click_pos.x - screen_rect.left()) as f64,
                (click_pos.y - screen_rect.top()) as f64,
            );

            // Compute collective bbox and handle positions for hit-testing
            let (collective_bbox, handle_screen_positions) = if !selected_path_ids.is_empty() {
                let mut union_bbox: Option<kurbo::Rect> = None;
                for id in selected_path_ids.iter() {
                    if let Some(path) = find_path_by_id(document, *id) {
                        let r = path.geometry.bounding_box();
                        union_bbox = Some(union_bbox.map_or(r, |a| a.union(r)));
                    }
                }
                if let Some(bbox) = union_bbox {
                    // Pad degenerate bbox so handles are reachable
                    let bbox = if bbox.width() > 0.0 && bbox.height() > 0.0 {
                        bbox
                    } else {
                        let cx = bbox.x0 + bbox.width() / 2.0;
                        let cy = bbox.y0 + bbox.height() / 2.0;
                        let w = bbox.width().max(MIN_BBOX_SIZE);
                        let h = bbox.height().max(MIN_BBOX_SIZE);
                        kurbo::Rect::from_center_size(kurbo::Point::new(cx, cy), (w, h))
                    };
                    let positions = compute_handle_screen_positions(
                        bbox, viewport, screen_rect, Affine::IDENTITY,
                    );
                    (Some(bbox), positions)
                } else {
                    (None, Vec::new())
                }
            } else {
                (None, Vec::new())
            };

            // 1. Check handle hit (including rotate)
            for (handle, pos) in &handle_screen_positions {
                let dist = ((click_pos.x - pos.x).powi(2) + (click_pos.y - pos.y).powi(2)).sqrt();
                let hit_radius = if *handle == HandlePosition::Rotate {
                    ROTATE_HANDLE_RADIUS + 4.0
                } else {
                    HANDLE_HIT_RADIUS
                };
                if dist < hit_radius {
                    if *handle == HandlePosition::Rotate {
                        interaction.mode = TransformMode::Rotating;
                        interaction.original_bbox = collective_bbox;
                    } else {
                        interaction.mode = TransformMode::Scaling { handle: handle.clone() };
                        interaction.original_bbox = collective_bbox;
                    }
                    return;
                }
            }

            // 2. Check move on selected paths
            for &pid in selected_path_ids.iter() {
                if let Some(path) = find_path_by_id(document, pid) {
                    if distance_to_path(&path.geometry, world_pos) < MOVE_TOLERANCE_PX / viewport.zoom {
                        interaction.mode = TransformMode::Moving;
                        return;
                    }
                }
            }

            // 3. Check vertex hit for point editing (single path only)
            if selected_path_ids.len() == 1 {
                let pid = selected_path_ids[0];
                if let Some(path) = find_path_by_id(document, pid) {
                    if let Some(idx) = hit_test_vertex_full(&path.geometry, world_pos, viewport.zoom) {
                        interaction.mode = TransformMode::EditingPoint { path_id: pid, element_idx: idx };
                        return;
                    }
                }
            }

            // 4. Hit test any path (with shift for multi-select toggle)
            let shift = ui.input(|i| i.modifiers.shift);
            let hit = hit_test(document, viewport, world_pos);
            if let Some(hid) = hit {
                if shift {
                    if let Some(pos) = selected_path_ids.iter().position(|&id| id == hid) {
                        selected_path_ids.remove(pos);
                    } else {
                        selected_path_ids.push(hid);
                    }
                } else {
                    *selected_path_ids = vec![hid];
                }

                // Auto-enter point editing for 2-element paths (single line segment)
                if selected_path_ids.len() == 1 {
                    if let Some(path) = find_path_by_id(document, hid) {
                        let elts = path.geometry.elements();
                        if elts.len() == 2 {
                            if let Some(idx) = hit_test_vertex_full(&path.geometry, world_pos, viewport.zoom) {
                                interaction.mode = TransformMode::EditingPoint { path_id: hid, element_idx: idx };
                                return;
                            }
                        }
                    }
                }

                interaction.mode = TransformMode::None;
            } else if response.drag_started() && !shift {
                interaction.mode = TransformMode::RectSelect;
            } else if !shift {
                selected_path_ids.clear();
            }
            return;
        }

        // ── Drag ongoing ──
        if response.dragged() && interaction.mode != TransformMode::None {
            if let Some(pos) = response.interact_pointer_pos() {
                interaction.drag_current_screen = pos;
            }
        }
    }
}

// ── Compute collective bounding box ──

pub fn compute_collective_bbox(document: &Document, ids: &[Uuid]) -> Option<kurbo::Rect> {
    let mut union_bbox: Option<kurbo::Rect> = None;
    for id in ids {
        if let Some(path) = find_path_by_id(document, *id) {
            let r = path.geometry.bounding_box();
            union_bbox = Some(union_bbox.map_or(r, |a| a.union(r)));
        }
    }
    union_bbox
}

// ── Helper functions ──

fn find_path_by_id<'a>(document: &'a Document, id: Uuid) -> Option<&'a crate::core::PlotPath> {
    for layer in &document.layers {
        for path in &layer.paths {
            if path.id == id {
                return Some(path);
            }
        }
    }
    None
}

// ── Hit-test ──

fn hit_test(document: &Document, viewport: &ViewportTransform, world_pos: kurbo::Point) -> Option<Uuid> {
    let layer = document.active_layer()?;
    let tolerance = SELECT_TOLERANCE_PX / viewport.zoom;
    let mut best: Option<(Uuid, f64)> = None;

    for path in &layer.paths {
        if !path.visible {
            continue;
        }
        let dist = distance_to_path(&path.geometry, world_pos);
        if dist < tolerance {
            match best {
                Some((_, d)) if dist < d => best = Some((path.id, dist)),
                None => best = Some((path.id, dist)),
                _ => {}
            }
        }
    }
    best.map(|(id, _)| id)
}

fn hit_test_vertex_full(path: &BezPath, world_pos: kurbo::Point, zoom: f64) -> Option<usize> {
    let radius = VERTEX_HIT_RADIUS_PX / zoom;
    let mut best: Option<(usize, f64)> = None;
    for (idx, el) in path.elements().iter().enumerate() {
        let pt = match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => *p,
            PathEl::QuadTo(_, p) => *p,
            PathEl::CurveTo(_, _, p) => *p,
            PathEl::ClosePath => continue,
        };
        let d = ((pt.x - world_pos.x).powi(2) + (pt.y - world_pos.y).powi(2)).sqrt();
        if d < radius {
            match best {
                Some((_, db)) if d < db => best = Some((idx, d)),
                None => best = Some((idx, d)),
                _ => {}
            }
        }
    }
    best.map(|(idx, _)| idx)
}

// ── Path intersection & distance ──

fn path_intersects_rect(path: &BezPath, rect: kurbo::Rect) -> bool {
    for el in path.elements() {
        let pt = match el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => *p,
            PathEl::QuadTo(_, p) => *p,
            PathEl::CurveTo(_, _, p) => *p,
            PathEl::ClosePath => continue,
        };
        if rect.contains(pt) {
            return true;
        }
    }
    let mut prev: Option<kurbo::Point> = None;
    for el in path.elements() {
        match el {
            PathEl::MoveTo(p) => { prev = Some(*p); }
            PathEl::LineTo(p) => {
                if let Some(a) = prev {
                    if segment_intersects_rect(a, *p, rect) { return true; }
                }
                prev = Some(*p);
            }
            PathEl::QuadTo(c, p) => {
                if let Some(a) = prev {
                    for i in 1..=8 {
                        let t1 = (i - 1) as f64 / 8.0;
                        let t2 = i as f64 / 8.0;
                        let s1 = eval_quad(a, *c, *p, t1);
                        let s2 = eval_quad(a, *c, *p, t2);
                        if segment_intersects_rect(s1, s2, rect) { return true; }
                    }
                }
                prev = Some(*p);
            }
            PathEl::CurveTo(c1, c2, p) => {
                if let Some(a) = prev {
                    for i in 1..=12 {
                        let t1 = (i - 1) as f64 / 12.0;
                        let t2 = i as f64 / 12.0;
                        let s1 = eval_cubic(a, *c1, *c2, *p, t1);
                        let s2 = eval_cubic(a, *c1, *c2, *p, t2);
                        if segment_intersects_rect(s1, s2, rect) { return true; }
                    }
                }
                prev = Some(*p);
            }
            PathEl::ClosePath => { prev = None; }
        }
    }
    false
}

fn segment_intersects_rect(a: kurbo::Point, b: kurbo::Point, rect: kurbo::Rect) -> bool {
    if rect.contains(a) || rect.contains(b) { return true; }
    let edges = [
        ((rect.x0, rect.y0), (rect.x1, rect.y0)),
        ((rect.x1, rect.y0), (rect.x1, rect.y1)),
        ((rect.x0, rect.y1), (rect.x1, rect.y1)),
        ((rect.x0, rect.y0), (rect.x0, rect.y1)),
    ];
    for (e1, e2) in edges {
        if segments_cross((a.x, a.y), (b.x, b.y), e1, e2) { return true; }
    }
    false
}

fn segments_cross(
    p1: (f64, f64), p2: (f64, f64),
    p3: (f64, f64), p4: (f64, f64),
) -> bool {
    let d1x = p2.0 - p1.0;
    let d1y = p2.1 - p1.1;
    let d2x = p4.0 - p3.0;
    let d2y = p4.1 - p3.1;
    let cross = d1x * d2y - d1y * d2x;
    if cross.abs() < 1e-12 { return false; }
    let dx = p3.0 - p1.0;
    let dy = p3.1 - p1.1;
    let t = (dx * d2y - dy * d2x) / cross;
    let u = (dx * d1y - dy * d1x) / cross;
    t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0
}

fn eval_quad(a: kurbo::Point, c: kurbo::Point, b: kurbo::Point, t: f64) -> kurbo::Point {
    let mt = 1.0 - t;
    kurbo::Point::new(
        mt * mt * a.x + 2.0 * mt * t * c.x + t * t * b.x,
        mt * mt * a.y + 2.0 * mt * t * c.y + t * t * b.y,
    )
}

fn eval_cubic(a: kurbo::Point, c1: kurbo::Point, c2: kurbo::Point, b: kurbo::Point, t: f64) -> kurbo::Point {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;
    kurbo::Point::new(
        mt3 * a.x + 3.0 * mt2 * t * c1.x + 3.0 * mt * t2 * c2.x + t3 * b.x,
        mt3 * a.y + 3.0 * mt2 * t * c1.y + 3.0 * mt * t2 * c2.y + t3 * b.y,
    )
}

fn point_to_segment(p: kurbo::Point, a: kurbo::Point, b: kurbo::Point) -> f64 {
    let abx = b.x - a.x;
    let aby = b.y - a.y;
    let len2 = abx * abx + aby * aby;
    if len2 < 1e-12 {
        return ((p.x - a.x).powi(2) + (p.y - a.y).powi(2)).sqrt();
    }
    let t = ((p.x - a.x) * abx + (p.y - a.y) * aby) / len2;
    let t = t.clamp(0.0, 1.0);
    let cx = a.x + t * abx;
    let cy = a.y + t * aby;
    ((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt()
}

fn distance_to_path(path: &BezPath, world_pos: kurbo::Point) -> f64 {
    let mut best = f64::MAX;
    let mut prev: Option<kurbo::Point> = None;
    for el in path.elements() {
        match el {
            PathEl::MoveTo(p) => {
                let d = ((p.x - world_pos.x).powi(2) + (p.y - world_pos.y).powi(2)).sqrt();
                if d < best { best = d; }
                prev = Some(*p);
            }
            PathEl::LineTo(p) => {
                if let Some(a) = prev {
                    let d = point_to_segment(world_pos, a, *p);
                    if d < best { best = d; }
                }
                prev = Some(*p);
            }
            PathEl::QuadTo(c, p) => {
                if let Some(a) = prev {
                    for i in 1..=16 {
                        let t = i as f64 / 16.0;
                        let s = eval_quad(a, *c, *p, t);
                        let d = ((s.x - world_pos.x).powi(2) + (s.y - world_pos.y).powi(2)).sqrt();
                        if d < best { best = d; }
                    }
                }
                prev = Some(*p);
            }
            PathEl::CurveTo(c1, c2, p) => {
                if let Some(a) = prev {
                    for i in 1..=24 {
                        let t = i as f64 / 24.0;
                        let s = eval_cubic(a, *c1, *c2, *p, t);
                        let d = ((s.x - world_pos.x).powi(2) + (s.y - world_pos.y).powi(2)).sqrt();
                        if d < best { best = d; }
                    }
                }
                prev = Some(*p);
            }
            PathEl::ClosePath => { prev = None; }
        }
    }
    best
}

// ── Drawing helpers ──

fn apply_affine_to_path(path: &BezPath, affine: Affine) -> BezPath {
    let mut new_path = BezPath::new();
    for el in path.elements() {
        match el {
            PathEl::MoveTo(p) => new_path.push(PathEl::MoveTo(affine * *p)),
            PathEl::LineTo(p) => new_path.push(PathEl::LineTo(affine * *p)),
            PathEl::QuadTo(p1, p2) => new_path.push(PathEl::QuadTo(affine * *p1, affine * *p2)),
            PathEl::CurveTo(p1, p2, p3) => {
                new_path.push(PathEl::CurveTo(affine * *p1, affine * *p2, affine * *p3));
            }
            PathEl::ClosePath => new_path.push(PathEl::ClosePath),
        }
    }
    new_path
}

fn bezpath_to_egui_fill_shapes(
    path: &BezPath,
    affine: Affine,
    screen_rect: Rect,
    color: Color32,
) -> Vec<Shape> {
    let transformed = apply_affine_to_path(path, affine);
    let mut shapes = Vec::new();
    let mut points: Vec<Pos2> = Vec::new();

    for el in transformed.elements() {
        match el {
            PathEl::MoveTo(p) => {
                if !points.is_empty() && points.len() > 2 {
                    shapes.push(Shape::convex_polygon(
                        points.clone(),
                        color,
                        Stroke::NONE,
                    ));
                }
                points.clear();
                points.push(Pos2::new(screen_rect.left() + p.x as f32, screen_rect.top() + p.y as f32));
            }
            PathEl::LineTo(p) => {
                points.push(Pos2::new(screen_rect.left() + p.x as f32, screen_rect.top() + p.y as f32));
            }
            PathEl::QuadTo(p1, p2) => {
                let start = *points.last().unwrap_or(&Pos2::ZERO);
                let cp1 = Pos2::new(screen_rect.left() + p1.x as f32, screen_rect.top() + p1.y as f32);
                let end = Pos2::new(screen_rect.left() + p2.x as f32, screen_rect.top() + p2.y as f32);
                for i in 1..=20 {
                    let t = i as f32 / 20.0;
                    let x = lerp(lerp(start.x, cp1.x, t), lerp(cp1.x, end.x, t), t);
                    let y = lerp(lerp(start.y, cp1.y, t), lerp(cp1.y, end.y, t), t);
                    points.push(Pos2::new(x, y));
                }
            }
            PathEl::CurveTo(p1, p2, p3) => {
                let start = *points.last().unwrap_or(&Pos2::ZERO);
                let cp1 = Pos2::new(screen_rect.left() + p1.x as f32, screen_rect.top() + p1.y as f32);
                let cp2 = Pos2::new(screen_rect.left() + p2.x as f32, screen_rect.top() + p2.y as f32);
                let end = Pos2::new(screen_rect.left() + p3.x as f32, screen_rect.top() + p3.y as f32);
                for i in 1..=30 {
                    let t = i as f32 / 30.0;
                    let x = cubic_bezier(start.x, cp1.x, cp2.x, end.x, t);
                    let y = cubic_bezier(start.y, cp1.y, cp2.y, end.y, t);
                    points.push(Pos2::new(x, y));
                }
            }
            PathEl::ClosePath => {
                if !points.is_empty() && points.len() > 2 {
                    points.push(points[0]);
                    shapes.push(Shape::convex_polygon(
                        points.clone(),
                        color,
                        Stroke::NONE,
                    ));
                }
                points.clear();
            }
        }
    }

    if !points.is_empty() && points.len() > 2 {
        shapes.push(Shape::convex_polygon(
            points,
            color,
            Stroke::NONE,
        ));
    }
    shapes
}

fn bezpath_to_egui_shapes(
    path: &BezPath,
    affine: Affine,
    screen_rect: Rect,
    color: Color32,
    stroke_width: f32,
) -> Vec<Shape> {
    let transformed = apply_affine_to_path(path, affine);
    let mut shapes = Vec::new();
    let mut points: Vec<Pos2> = Vec::new();

    for el in transformed.elements() {
        match el {
            PathEl::MoveTo(p) => {
                if !points.is_empty() && points.len() > 1 {
                    shapes.push(Shape::line(points.clone(), Stroke::new(stroke_width, color)));
                }
                points.clear();
                points.push(Pos2::new(screen_rect.left() + p.x as f32, screen_rect.top() + p.y as f32));
            }
            PathEl::LineTo(p) => {
                points.push(Pos2::new(screen_rect.left() + p.x as f32, screen_rect.top() + p.y as f32));
            }
            PathEl::QuadTo(p1, p2) => {
                let start = *points.last().unwrap_or(&Pos2::ZERO);
                let cp1 = Pos2::new(screen_rect.left() + p1.x as f32, screen_rect.top() + p1.y as f32);
                let end = Pos2::new(screen_rect.left() + p2.x as f32, screen_rect.top() + p2.y as f32);
                for i in 1..=20 {
                    let t = i as f32 / 20.0;
                    let x = lerp(lerp(start.x, cp1.x, t), lerp(cp1.x, end.x, t), t);
                    let y = lerp(lerp(start.y, cp1.y, t), lerp(cp1.y, end.y, t), t);
                    points.push(Pos2::new(x, y));
                }
            }
            PathEl::CurveTo(p1, p2, p3) => {
                let start = *points.last().unwrap_or(&Pos2::ZERO);
                let cp1 = Pos2::new(screen_rect.left() + p1.x as f32, screen_rect.top() + p1.y as f32);
                let cp2 = Pos2::new(screen_rect.left() + p2.x as f32, screen_rect.top() + p2.y as f32);
                let end = Pos2::new(screen_rect.left() + p3.x as f32, screen_rect.top() + p3.y as f32);
                for i in 1..=30 {
                    let t = i as f32 / 30.0;
                    let x = cubic_bezier(start.x, cp1.x, cp2.x, end.x, t);
                    let y = cubic_bezier(start.y, cp1.y, cp2.y, end.y, t);
                    points.push(Pos2::new(x, y));
                }
            }
            PathEl::ClosePath => {
                if !points.is_empty() && points.len() > 1 {
                    points.push(points[0]);
                    shapes.push(Shape::line(points.clone(), Stroke::new(stroke_width, color)));
                }
                points.clear();
            }
        }
    }

    if !points.is_empty() && points.len() > 1 {
        shapes.push(Shape::line(points, Stroke::new(stroke_width, color)));
    }
    shapes
}

fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    p0 * mt3 + 3.0 * p1 * mt2 * t + 3.0 * p2 * mt * t2 + p3 * t3
}

// ── Paper border and grid ──

impl InteractiveCanvas {
    fn draw_paper_border(
        painter: &egui::Painter,
        screen_rect: Rect,
        viewport: &ViewportTransform,
        document: &Document,
    ) {
        let w = document.paper_width_cm * CM_TO_UNITS;
        let h = document.paper_height_cm * CM_TO_UNITS;
        let half_w = w / 2.0;
        let half_h = h / 2.0;

        let world_corners = [
            kurbo::Point::new(-half_w, -half_h),
            kurbo::Point::new(half_w, -half_h),
            kurbo::Point::new(half_w, half_h),
            kurbo::Point::new(-half_w, half_h),
        ];

        let affine = viewport.affine();
        let screen_corners: Vec<Pos2> = world_corners
            .iter()
            .map(|p| {
                let t = affine * *p;
                Pos2::new(screen_rect.left() + t.x as f32, screen_rect.top() + t.y as f32)
            })
            .collect();

        if let [a, b, c, d] = screen_corners.as_slice() {
            let corners = vec![*a, *b, *c, *d, *a];
            let stroke = Stroke::new(1.0, Color32::from_rgb(100, 140, 180));
            if document.border_dashed {
                painter.add(Shape::dashed_line(&corners, stroke, 6.0, 4.0));
            } else {
                painter.add(Shape::line(corners, stroke));
            }
        }
    }

    fn draw_grid(
        painter: &egui::Painter,
        rect: Rect,
        viewport: &ViewportTransform,
    ) {
        let grid_size: f64 = 50.0;
        let zoom = viewport.zoom;
        let spacing = (grid_size * zoom).max(10.0);
        if spacing < 5.0 { return; }

        let grid_color = Color32::from_gray(48);
        let stroke = Stroke::new(0.5, grid_color);

        let world_min = viewport.screen_to_world(rect.left() as f64, rect.top() as f64);
        let world_max = viewport.screen_to_world(rect.right() as f64, rect.bottom() as f64);

        let start_x = (world_min.x / grid_size).floor() as i32;
        let start_y = (world_min.y / grid_size).floor() as i32;
        let end_x = (world_max.x / grid_size).ceil() as i32;
        let end_y = (world_max.y / grid_size).ceil() as i32;

        for x in start_x..=end_x {
            let wx = x as f64 * grid_size;
            let sp = viewport.world_to_screen(wx, 0.0);
            let sx = sp.x as f32;
            if sx >= rect.left() && sx <= rect.right() {
                painter.vline(sx, rect.y_range(), stroke);
            }
        }
        for y in start_y..=end_y {
            let wy = y as f64 * grid_size;
            let sp = viewport.world_to_screen(0.0, wy);
            let sy = sp.y as f32;
            if sy >= rect.top() && sy <= rect.bottom() {
                painter.hline(rect.x_range(), sy, stroke);
            }
        }
    }
}

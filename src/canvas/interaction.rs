use eframe::egui::Pos2;
use kurbo::{Affine, Rect, Vec2};
use uuid::Uuid;

use crate::canvas::ViewportTransform;

#[derive(Clone, Debug, PartialEq)]
pub enum HandlePosition {
    TopLeft, TopCenter, TopRight,
    MiddleLeft, MiddleRight,
    BottomLeft, BottomCenter, BottomRight,
    Rotate,
}

impl HandlePosition {
    pub fn opposite(&self) -> Self {
        use HandlePosition::*;
        match self {
            TopLeft => BottomRight,
            TopCenter => BottomCenter,
            TopRight => BottomLeft,
            MiddleLeft => MiddleRight,
            MiddleRight => MiddleLeft,
            BottomLeft => TopRight,
            BottomCenter => TopCenter,
            BottomRight => TopLeft,
            Rotate => Rotate,
        }
    }

    pub fn corner_point(&self, bbox: Rect) -> (f64, f64) {
        use HandlePosition::*;
        match self {
            TopLeft => (bbox.x0, bbox.y0),
            TopCenter => (bbox.x0 + bbox.width() / 2.0, bbox.y0),
            TopRight => (bbox.x1, bbox.y0),
            MiddleLeft => (bbox.x0, bbox.y0 + bbox.height() / 2.0),
            MiddleRight => (bbox.x1, bbox.y0 + bbox.height() / 2.0),
            BottomLeft => (bbox.x0, bbox.y1),
            BottomCenter => (bbox.x0 + bbox.width() / 2.0, bbox.y1),
            BottomRight => (bbox.x1, bbox.y1),
            Rotate => (bbox.x0 + bbox.width() / 2.0, bbox.y0 - bbox.height() * 0.15),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransformMode {
    None,
    Moving,
    Scaling { handle: HandlePosition },
    EditingPoint { path_id: Uuid, element_idx: usize },
    RectSelect,
    Rotating,
}

#[derive(Clone, Debug)]
pub enum PendingAction {
    TranslatePaths { ids: Vec<Uuid>, dx: f64, dy: f64 },
    ScalePaths { ids: Vec<Uuid>, cx: f64, cy: f64, sx: f64, sy: f64 },
    EditPoint { id: Uuid, element_idx: usize, new_x: f64, new_y: f64 },
    RotatePaths { ids: Vec<Uuid>, cx: f64, cy: f64, angle_rad: f64 },
}

#[derive(Clone, Debug)]
pub struct InteractionState {
    pub mode: TransformMode,
    pub pending_action: Option<PendingAction>,
    pub drag_start_screen: Pos2,
    pub drag_current_screen: Pos2,
    pub original_bbox: Option<Rect>,
    #[allow(dead_code)]
    pub drag_start_rot_rad: f64,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self {
            mode: TransformMode::None,
            pending_action: None,
            drag_start_screen: Pos2::ZERO,
            drag_current_screen: Pos2::ZERO,
            original_bbox: None,
            drag_start_rot_rad: 0.0,
        }
    }
}

/// Returns the 9 handle positions (8 resize + 1 rotate) in screen coords.
/// `visual_affine` is the interaction's live preview transform (from `visual_transform`).
pub fn compute_handle_screen_positions(
    bbox: Rect,
    viewport: &ViewportTransform,
    screen_rect: eframe::egui::Rect,
    visual_affine: Affine,
) -> Vec<(HandlePosition, Pos2)> {
    let transform_pt = |wx: f64, wy: f64| -> Pos2 {
        let p = kurbo::Point::new(wx, wy);
        let t = viewport.affine() * visual_affine * p;
        Pos2::new(screen_rect.left() + t.x as f32, screen_rect.top() + t.y as f32)
    };

    let c0 = transform_pt(bbox.x0, bbox.y0);
    let c1 = transform_pt(bbox.x1, bbox.y0);
    let c2 = transform_pt(bbox.x1, bbox.y1);
    let c3 = transform_pt(bbox.x0, bbox.y1);

    let mid0 = midpoint(c0, c1);
    let mid1 = midpoint(c1, c2);
    let mid2 = midpoint(c2, c3);
    let mid3 = midpoint(c3, c0);

    // Rotate handle: above top center
    let rotate_offset = 20.0_f32;
    let rotate_pos = Pos2::new(mid0.x, mid0.y - rotate_offset);

    vec![
        (HandlePosition::TopLeft, c0),
        (HandlePosition::TopCenter, mid0),
        (HandlePosition::TopRight, c1),
        (HandlePosition::MiddleLeft, mid3),
        (HandlePosition::MiddleRight, mid1),
        (HandlePosition::BottomLeft, c3),
        (HandlePosition::BottomCenter, mid2),
        (HandlePosition::BottomRight, c2),
        (HandlePosition::Rotate, rotate_pos),
    ]
}

fn midpoint(a: Pos2, b: Pos2) -> Pos2 {
    Pos2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}

/// Compute the current angle (in radians) from drag_start_screen to drag_current_screen
/// around the center of the original_bbox, in world space.
fn compute_drag_angle(
    interaction: &InteractionState,
    viewport: &ViewportTransform,
    screen_rect: eframe::egui::Rect,
) -> f64 {
    let bbox = match interaction.original_bbox {
        Some(b) => b,
        None => return 0.0,
    };
    let center_x = bbox.x0 + bbox.width() / 2.0;
    let center_y = bbox.y0 + bbox.height() / 2.0;

    let start_w = viewport.screen_to_world(
        (interaction.drag_start_screen.x - screen_rect.left()) as f64,
        (interaction.drag_start_screen.y - screen_rect.top()) as f64,
    );
    let cur_w = viewport.screen_to_world(
        (interaction.drag_current_screen.x - screen_rect.left()) as f64,
        (interaction.drag_current_screen.y - screen_rect.top()) as f64,
    );

    let start_angle = (start_w.y - center_y).atan2(start_w.x - center_x);
    let cur_angle = (cur_w.y - center_y).atan2(cur_w.x - center_x);
    cur_angle - start_angle
}

impl InteractionState {
    /// Returns an Affine that represents the live preview of the current interaction.
    /// This is applied to both the rendered paths and the selection handles.
    pub fn visual_transform(&self, viewport: &ViewportTransform, screen_rect: eframe::egui::Rect) -> Affine {
        match &self.mode {
            TransformMode::None
            | TransformMode::RectSelect
            | TransformMode::EditingPoint { .. } => Affine::IDENTITY,

            TransformMode::Moving => {
                let start_w = viewport.screen_to_world(
                    (self.drag_start_screen.x - screen_rect.left()) as f64,
                    (self.drag_start_screen.y - screen_rect.top()) as f64,
                );
                let cur_w = viewport.screen_to_world(
                    (self.drag_current_screen.x - screen_rect.left()) as f64,
                    (self.drag_current_screen.y - screen_rect.top()) as f64,
                );
                Affine::translate(Vec2::new(cur_w.x - start_w.x, cur_w.y - start_w.y))
            }

            TransformMode::Scaling { handle } => {
                let bbox = match self.original_bbox {
                    Some(b) => b,
                    None => return Affine::IDENTITY,
                };

                let (fx, fy) = handle.opposite().corner_point(bbox);
                let (dx_orig, dy_orig) = handle.corner_point(bbox);

                let cur_w = viewport.screen_to_world(
                    (self.drag_current_screen.x - screen_rect.left()) as f64,
                    (self.drag_current_screen.y - screen_rect.top()) as f64,
                );

                let sx = if (dx_orig - fx).abs() > 1e-10 {
                    (cur_w.x - fx) / (dx_orig - fx)
                } else {
                    1.0
                };
                let sy = if (dy_orig - fy).abs() > 1e-10 {
                    (cur_w.y - fy) / (dy_orig - fy)
                } else {
                    1.0
                };

                // scale about the fixed corner
                Affine::translate(Vec2::new(fx, fy))
                    * Affine::scale_non_uniform(sx, sy)
                    * Affine::translate(Vec2::new(-fx, -fy))
            }

            TransformMode::Rotating => {
                let angle = compute_drag_angle(self, viewport, screen_rect);
                if angle.abs() < 0.001 {
                    return Affine::IDENTITY;
                }
                let bbox = match self.original_bbox {
                    Some(b) => b,
                    None => return Affine::IDENTITY,
                };
                let cx = bbox.x0 + bbox.width() / 2.0;
                let cy = bbox.y0 + bbox.height() / 2.0;
                Affine::translate(Vec2::new(cx, cy))
                    * Affine::rotate(angle)
                    * Affine::translate(Vec2::new(-cx, -cy))
            }
        }
    }

    /// Compute the pending action for the current interaction (called on drag_stopped).
    pub fn compute_pending_action(&self, viewport: &ViewportTransform, screen_rect: eframe::egui::Rect, selected_path_ids: &[Uuid]) -> Option<PendingAction> {
        match &self.mode {
            TransformMode::Moving => {
                let visual = self.visual_transform(viewport, screen_rect);
                // Extract translation from affine
                let tx = visual.as_coeffs()[2];
                let ty = visual.as_coeffs()[5];
                if tx.abs() > 0.01 || ty.abs() > 0.01 {
                    Some(PendingAction::TranslatePaths {
                        ids: selected_path_ids.to_vec(),
                        dx: tx,
                        dy: ty,
                    })
                } else {
                    None
                }
            }
            TransformMode::Scaling { .. } => {
                let bbox = match self.original_bbox {
                    Some(b) => b,
                    None => return None,
                };
                // Reconstruct handle from mode
                let handle = match &self.mode {
                    TransformMode::Scaling { handle } => handle.clone(),
                    _ => return None,
                };
                let (fx, fy) = handle.opposite().corner_point(bbox);
                let (dx_orig, dy_orig) = handle.corner_point(bbox);
                let cur_w = viewport.screen_to_world(
                    (self.drag_current_screen.x - screen_rect.left()) as f64,
                    (self.drag_current_screen.y - screen_rect.top()) as f64,
                );
                let sx = if (dx_orig - fx).abs() > 1e-10 {
                    (cur_w.x - fx) / (dx_orig - fx)
                } else {
                    1.0
                };
                let sy = if (dy_orig - fy).abs() > 1e-10 {
                    (cur_w.y - fy) / (dy_orig - fy)
                } else {
                    1.0
                };
                if sx != 1.0 || sy != 1.0 {
                    Some(PendingAction::ScalePaths {
                        ids: selected_path_ids.to_vec(),
                        cx: fx, cy: fy, sx, sy,
                    })
                } else {
                    None
                }
            }
            TransformMode::Rotating => {
                let angle = compute_drag_angle(self, viewport, screen_rect);
                if angle.abs() > 0.005 {
                    let bbox = self.original_bbox.unwrap();
                    let cx = bbox.x0 + bbox.width() / 2.0;
                    let cy = bbox.y0 + bbox.height() / 2.0;
                    Some(PendingAction::RotatePaths {
                        ids: selected_path_ids.to_vec(),
                        cx, cy, angle_rad: angle,
                    })
                } else {
                    None
                }
            }
            TransformMode::EditingPoint { path_id, element_idx } => {
                let cur_w = viewport.screen_to_world(
                    (self.drag_current_screen.x - screen_rect.left()) as f64,
                    (self.drag_current_screen.y - screen_rect.top()) as f64,
                );
                let start_w = viewport.screen_to_world(
                    (self.drag_start_screen.x - screen_rect.left()) as f64,
                    (self.drag_start_screen.y - screen_rect.top()) as f64,
                );
                if (cur_w.x - start_w.x).abs() > 0.5 || (cur_w.y - start_w.y).abs() > 0.5 {
                    Some(PendingAction::EditPoint {
                        id: *path_id,
                        element_idx: *element_idx,
                        new_x: cur_w.x,
                        new_y: cur_w.y,
                    })
                } else {
                    None
                }
            }
            TransformMode::RectSelect | TransformMode::None => None,
        }
    }
}

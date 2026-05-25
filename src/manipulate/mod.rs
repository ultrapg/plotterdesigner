use kurbo::Affine;
use uuid::Uuid;

use crate::core::Document;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ManipulateState {
    pub scale: f64,
    pub rotation_deg: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

impl Default for ManipulateState {
    fn default() -> Self {
        Self {
            scale: 1.0,
            rotation_deg: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

#[allow(dead_code)]
fn apply_affine_to_path_elements(
    path: &mut kurbo::BezPath,
    affine: Affine,
) {
    let elements: Vec<kurbo::PathEl> = path.elements().to_vec();
    let mut new_path = kurbo::BezPath::new();
    for el in elements {
        match el {
            kurbo::PathEl::MoveTo(p) => new_path.push(kurbo::PathEl::MoveTo(affine * p)),
            kurbo::PathEl::LineTo(p) => new_path.push(kurbo::PathEl::LineTo(affine * p)),
            kurbo::PathEl::QuadTo(p1, p2) => {
                new_path.push(kurbo::PathEl::QuadTo(affine * p1, affine * p2));
            }
            kurbo::PathEl::CurveTo(p1, p2, p3) => {
                new_path.push(kurbo::PathEl::CurveTo(
                    affine * p1,
                    affine * p2,
                    affine * p3,
                ));
            }
            kurbo::PathEl::ClosePath => new_path.push(kurbo::PathEl::ClosePath),
        }
    }
    *path = new_path;
}

#[allow(dead_code)]
pub fn apply_transform_to_selection(
    document: &mut Document,
    selected: &[Uuid],
    affine: &Affine,
) {
    for layer in &mut document.layers {
        for path in &mut layer.paths {
            if selected.contains(&path.id) {
                apply_affine_to_path_elements(&mut path.geometry, *affine);
            }
        }
    }
}

pub fn translate_path(document: &mut Document, id: Uuid, dx: f64, dy: f64) {
    let affine = Affine::translate((dx, dy));
    for layer in &mut document.layers {
        for path in &mut layer.paths {
            if path.id == id {
                apply_affine_to_path_elements(&mut path.geometry, affine);
                return;
            }
        }
    }
}

pub fn scale_path(document: &mut Document, id: Uuid, cx: f64, cy: f64, sx: f64, sy: f64) {
    let affine = Affine::translate((cx, cy))
        * Affine::scale_non_uniform(sx, sy)
        * Affine::translate((-cx, -cy));
    for layer in &mut document.layers {
        for path in &mut layer.paths {
            if path.id == id {
                apply_affine_to_path_elements(&mut path.geometry, affine);
                return;
            }
        }
    }
}

pub fn edit_point(document: &mut Document, id: Uuid, element_idx: usize, new_x: f64, new_y: f64) {
    for layer in &mut document.layers {
        for path in &mut layer.paths {
            if path.id == id {
                let elements: Vec<kurbo::PathEl> = path.geometry.elements().to_vec();
                if element_idx >= elements.len() {
                    return;
                }
                let mut new_elements = elements.clone();
                new_elements[element_idx] = match elements[element_idx] {
                    kurbo::PathEl::MoveTo(_) => kurbo::PathEl::MoveTo(kurbo::Point::new(new_x, new_y)),
                    kurbo::PathEl::LineTo(_) => kurbo::PathEl::LineTo(kurbo::Point::new(new_x, new_y)),
                    kurbo::PathEl::QuadTo(cp, _) => kurbo::PathEl::QuadTo(cp, kurbo::Point::new(new_x, new_y)),
                    kurbo::PathEl::CurveTo(cp1, cp2, _) => kurbo::PathEl::CurveTo(cp1, cp2, kurbo::Point::new(new_x, new_y)),
                    kurbo::PathEl::ClosePath => kurbo::PathEl::ClosePath,
                };
                path.geometry = kurbo::BezPath::from_vec(new_elements);
                return;
            }
        }
    }
}

pub fn rotate_paths(document: &mut Document, ids: &[Uuid], cx: f64, cy: f64, angle_rad: f64) {
    let (s, c) = angle_rad.sin_cos();
    let affine = Affine::translate((cx, cy)) * Affine::new([c, s, -s, c, 0.0, 0.0]) * Affine::translate((-cx, -cy));
    for layer in &mut document.layers {
        for path in &mut layer.paths {
            if ids.contains(&path.id) {
                apply_affine_to_path_elements(&mut path.geometry, affine);
            }
        }
    }
}

pub fn duplicate_selection(document: &mut Document, selected: &[Uuid]) {
    for layer in &mut document.layers {
        let mut new_paths = Vec::new();
        for path in &layer.paths {
            if selected.contains(&path.id) {
                let mut dup = path.clone();
                dup.id = Uuid::new_v4();
                dup.name = format!("{} (copy)", path.name);
                new_paths.push(dup);
            }
        }
        layer.paths.append(&mut new_paths);
    }
}

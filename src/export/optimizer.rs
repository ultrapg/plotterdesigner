use kurbo::{BezPath, PathEl, Point};

use crate::core::Document;

#[derive(Clone, Debug)]
pub struct OptimizedPath {
    pub path: BezPath,
    pub layer_name: String,
    pub stroke_color: String,
    pub stroke_width: f64,
}

pub struct PathOptimizer;

impl PathOptimizer {
    pub fn optimize(document: &Document) -> Vec<OptimizedPath> {
        let mut result = Vec::new();

        for layer in &document.layers {
            if !layer.visible {
                continue;
            }

            let mut layer_paths: Vec<BezPath> = layer
                .paths
                .iter()
                .filter(|p| p.visible)
                .map(|p| p.geometry.clone())
                .collect();

            // Sort paths by nearest-neighbor (greedy TSP)
            PathOptimizer::sort_by_nearest_neighbor(&mut layer_paths);

            for path in layer_paths {
                let color = Self::color_to_svg_hex(layer.pen.color);
                result.push(OptimizedPath {
                    path,
                    layer_name: layer.name.clone(),
                    stroke_color: color,
                    stroke_width: layer.pen.width_mm,
                });
            }
        }

        result
    }

    fn sort_by_nearest_neighbor(paths: &mut Vec<BezPath>) {
        if paths.len() <= 1 {
            return;
        }

        let mut sorted: Vec<BezPath> = Vec::with_capacity(paths.len());
        let mut used = vec![false; paths.len()];

        sorted.push(paths[0].clone());
        used[0] = true;

        let mut last_end = Self::path_endpoint(&paths[0]);

        for _ in 1..paths.len() {
            let mut best_idx = None;
            let mut best_dist = f64::MAX;
            let mut should_reverse = false;

            for (i, path) in paths.iter().enumerate() {
                if used[i] {
                    continue;
                }
                let start = Self::path_startpoint(path);
                let end = Self::path_endpoint(path);

                let dist_start = last_end.distance(start);
                let dist_end = last_end.distance(end);

                if dist_start < best_dist {
                    best_dist = dist_start;
                    best_idx = Some(i);
                    should_reverse = false;
                }
                if dist_end < best_dist {
                    best_dist = dist_end;
                    best_idx = Some(i);
                    should_reverse = true;
                }
            }

            if let Some(idx) = best_idx {
                let mut path = paths[idx].clone();
                if should_reverse {
                    path = Self::reverse_path(&path);
                }
                last_end = Self::path_endpoint(&path);
                sorted.push(path);
                used[idx] = true;
            }
        }

        paths.clear();
        paths.extend(sorted);
    }

    fn path_startpoint(path: &BezPath) -> Point {
        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => return *p,
                PathEl::QuadTo(_, p) => return *p,
                PathEl::CurveTo(_, _, p) => return *p,
                PathEl::ClosePath => break,
            }
        }
        Point::ZERO
    }

    fn path_endpoint(path: &BezPath) -> Point {
        let mut last = Point::ZERO;
        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) | PathEl::LineTo(p) => last = *p,
                PathEl::QuadTo(_, p) => last = *p,
                PathEl::CurveTo(_, _, p) => last = *p,
                PathEl::ClosePath => {}
            }
        }
        last
    }

    fn reverse_path(path: &BezPath) -> BezPath {
        let mut new_path = BezPath::new();
        let points = path.elements().to_vec();

        if points.len() < 2 {
            return path.clone();
        }

        let end = Self::path_endpoint(path);
        new_path.push(PathEl::MoveTo(end));

        for el in points[1..].iter().rev() {
            match el {
                PathEl::LineTo(p) => new_path.push(PathEl::LineTo(*p)),
                PathEl::QuadTo(p1, p2) => new_path.push(PathEl::QuadTo(*p2, *p1)),
                PathEl::CurveTo(p1, p2, p3) => new_path.push(PathEl::CurveTo(*p3, *p2, *p1)),
                _ => new_path.push(*el),
            }
        }

        new_path
    }

    fn color_to_svg_hex(color: eframe::egui::Color32) -> String {
        format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
    }
}

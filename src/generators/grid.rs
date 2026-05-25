use kurbo::{BezPath, PathEl, Point};

use super::{Generator, GeneratorKind, GeneratorParams};
use crate::core::PlotPath;

#[derive(Clone, Debug)]
pub struct GridParams {
    pub cols: u32,
    pub rows: u32,
    pub width: f64,
    pub height: f64,
}

impl Default for GridParams {
    fn default() -> Self {
        Self {
            cols: 10,
            rows: 10,
            width: 400.0,
            height: 400.0,
        }
    }
}

pub struct GridGenerator;

impl Generator for GridGenerator {
    fn generate(&self, params: &GeneratorParams) -> Vec<PlotPath> {
        let p = &params.grid;
        let mut paths = Vec::new();
        let half_w = p.width / 2.0;
        let half_h = p.height / 2.0;

        // Vertical lines
        for i in 0..=p.cols {
            let x = -half_w + (i as f64 / p.cols as f64) * p.width;
            let mut bez = BezPath::new();
            bez.push(PathEl::MoveTo(Point::new(x, -half_h)));
            bez.push(PathEl::LineTo(Point::new(x, half_h)));
            paths.push(PlotPath::new(bez, false).with_name(format!("Grid V{}", i)));
        }

        // Horizontal lines
        for i in 0..=p.rows {
            let y = -half_h + (i as f64 / p.rows as f64) * p.height;
            let mut bez = BezPath::new();
            bez.push(PathEl::MoveTo(Point::new(-half_w, y)));
            bez.push(PathEl::LineTo(Point::new(half_w, y)));
            paths.push(PlotPath::new(bez, false).with_name(format!("Grid H{}", i)));
        }

        paths
    }

    fn kind(&self) -> GeneratorKind {
        GeneratorKind::Grid
    }

    fn name(&self) -> &'static str {
        "Grid"
    }
}

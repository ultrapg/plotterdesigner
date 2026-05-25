use kurbo::{BezPath, PathEl, Point};
use std::f64::consts::{FRAC_PI_3, TAU};

use super::{Generator, GeneratorKind, GeneratorParams};
use crate::core::PlotPath;

#[derive(Clone, Debug)]
pub struct HoneycombParams {
    pub cell_radius: f64,
    pub cols: u32,
    pub rows: u32,
}

impl Default for HoneycombParams {
    fn default() -> Self {
        Self {
            cell_radius: 30.0,
            cols: 8,
            rows: 8,
        }
    }
}

pub struct HoneycombGenerator;

impl Generator for HoneycombGenerator {
    fn generate(&self, params: &GeneratorParams) -> Vec<PlotPath> {
        let p = &params.honeycomb;
        let mut paths = Vec::new();
        let hex_w = p.cell_radius * (FRAC_PI_3 / 2.0).cos() * 2.0; // width of hex = sqrt(3)*R
        let hex_h = p.cell_radius * 1.5; // height of hex = 1.5*R

        for row in 0..p.rows {
            let cols_in_row = if row % 2 == 0 { p.cols } else { p.cols - 1 };
            let x_offset = if row % 2 == 0 { 0.0 } else { hex_w / 2.0 };

            for col in 0..cols_in_row {
                let cx = col as f64 * hex_w + x_offset;
                let cy = row as f64 * hex_h;
                paths.push(make_hexagon(cx, cy, p.cell_radius));
            }
        }

        paths
    }

    fn kind(&self) -> GeneratorKind {
        GeneratorKind::Honeycomb
    }

    fn name(&self) -> &'static str {
        "Honeycomb"
    }
}

fn make_hexagon(cx: f64, cy: f64, radius: f64) -> PlotPath {
    let mut bez = BezPath::new();
    for i in 0..6 {
        let angle = i as f64 * TAU / 6.0 - FRAC_PI_3 / 2.0;
        let x = cx + radius * angle.cos();
        let y = cy + radius * angle.sin();
        if i == 0 {
            bez.push(PathEl::MoveTo(Point::new(x, y)));
        } else {
            bez.push(PathEl::LineTo(Point::new(x, y)));
        }
    }
    bez.push(PathEl::ClosePath);
    PlotPath::new(bez, true)
}

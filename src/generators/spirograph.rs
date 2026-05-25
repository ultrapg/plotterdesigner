use kurbo::{BezPath, PathEl};
use std::f64::consts::TAU;

use super::{Generator, GeneratorKind, GeneratorParams};
use crate::core::PlotPath;

#[derive(Clone, Debug)]
pub struct SpirographParams {
    pub outer_radius: f64,
    pub inner_radius: f64,
    pub pen_offset: f64,
    pub revolutions: f64,
    pub num_samples: u32,
}

impl Default for SpirographParams {
    fn default() -> Self {
        Self {
            outer_radius: 200.0,
            inner_radius: 80.0,
            pen_offset: 60.0,
            revolutions: 6.0,
            num_samples: 2000,
        }
    }
}

pub struct SpirographGenerator;

impl Generator for SpirographGenerator {
    fn generate(&self, params: &GeneratorParams) -> Vec<PlotPath> {
        let p = &params.spirograph;
        let mut path = BezPath::new();
        let mut first = true;

        let total = p.revolutions * TAU;
        let step = total / p.num_samples as f64;

        for i in 0..=p.num_samples {
            let t = i as f64 * step;
            let r_diff = p.outer_radius - p.inner_radius;
            let ratio = if p.inner_radius != 0.0 {
                r_diff / p.inner_radius
            } else {
                0.0
            };
            let x = r_diff * t.cos() + p.pen_offset * (ratio * t).cos();
            let y = r_diff * t.sin() - p.pen_offset * (ratio * t).sin();

            if first {
                path.push(PathEl::MoveTo(kurbo::Point::new(x, y)));
                first = false;
            } else {
                path.push(PathEl::LineTo(kurbo::Point::new(x, y)));
            }
        }

        vec![PlotPath::new(path, false).with_name("Spirograph")]
    }

    fn kind(&self) -> GeneratorKind {
        GeneratorKind::Spirograph
    }

    fn name(&self) -> &'static str {
        "Spirograph"
    }
}

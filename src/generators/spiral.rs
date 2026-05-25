use kurbo::{BezPath, PathEl};
use std::f64::consts::TAU;

use super::{Generator, GeneratorKind, GeneratorParams};
use crate::core::PlotPath;

#[derive(Clone, Debug)]
pub struct SpiralParams {
    pub turns: f64,
    pub max_radius: f64,
    pub growth: f64,
    pub num_samples: u32,
}

impl Default for SpiralParams {
    fn default() -> Self {
        Self {
            turns: 8.0,
            max_radius: 200.0,
            growth: 1.0,
            num_samples: 2000,
        }
    }
}

pub struct SpiralGenerator;

impl Generator for SpiralGenerator {
    fn generate(&self, params: &GeneratorParams) -> Vec<PlotPath> {
        let p = &params.spiral;
        let mut path = BezPath::new();
        let mut first = true;

        let total_angle = p.turns * TAU;
        let step = total_angle / p.num_samples as f64;

        for i in 0..=p.num_samples {
            let t = i as f64 * step;
            let frac = t / total_angle;
            let r = p.max_radius * frac.powf(p.growth);
            let x = r * t.cos();
            let y = r * t.sin();

            if first {
                path.push(PathEl::MoveTo(kurbo::Point::new(x, y)));
                first = false;
            } else {
                path.push(PathEl::LineTo(kurbo::Point::new(x, y)));
            }
        }

        vec![PlotPath::new(path, false).with_name("Spiral")]
    }

    fn kind(&self) -> GeneratorKind {
        GeneratorKind::Spiral
    }

    fn name(&self) -> &'static str {
        "Spiral"
    }
}

use kurbo::{BezPath, PathEl, Point};
use std::f64::consts::TAU;

use super::{Generator, GeneratorKind, GeneratorParams};
use crate::core::PlotPath;

#[derive(Clone, Debug)]
pub struct WaveParams {
    pub amplitude: f64,
    pub frequency: f64,
    pub num_waves: u32,
    pub width: f64,
    pub height: f64,
    pub phase_shift: f64,
}

impl Default for WaveParams {
    fn default() -> Self {
        Self {
            amplitude: 60.0,
            frequency: 3.0,
            num_waves: 5,
            width: 400.0,
            height: 300.0,
            phase_shift: 0.0,
        }
    }
}

pub struct WaveGenerator;

impl Generator for WaveGenerator {
    fn generate(&self, params: &GeneratorParams) -> Vec<PlotPath> {
        let p = &params.wave;
        let mut paths = Vec::new();
        let samples = params.resolution as usize;

        for wave_idx in 0..p.num_waves {
            let mut bez = BezPath::new();
            let y_offset = -p.height / 2.0
                + (wave_idx as f64 + 0.5) * p.height / p.num_waves as f64;

            for i in 0..=samples {
                let t = i as f64 / samples as f64;
                let x = -p.width / 2.0 + t * p.width;
                let angle = t * TAU * p.frequency + p.phase_shift;
                let y = y_offset + p.amplitude * angle.sin();

                if i == 0 {
                    bez.push(PathEl::MoveTo(Point::new(x, y)));
                } else {
                    bez.push(PathEl::LineTo(Point::new(x, y)));
                }
            }

            paths.push(PlotPath::new(bez, false).with_name(format!("Wave {}", wave_idx + 1)));
        }

        paths
    }

    fn kind(&self) -> GeneratorKind {
        GeneratorKind::Wave
    }

    fn name(&self) -> &'static str {
        "Wave"
    }
}

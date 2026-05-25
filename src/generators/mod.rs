pub mod spirograph;
pub mod l_system;
pub mod voronoi;
pub mod wave;
pub mod grid;
pub mod honeycomb;
pub mod spiral;

use crate::core::PlotPath;

#[derive(Clone, Debug)]
pub struct AddPrimitiveParams {
    pub line_length: f64,
    pub line_angle_deg: f64,
    pub rect_width: f64,
    pub rect_height: f64,
    pub ellipse_rx: f64,
    pub ellipse_ry: f64,
    pub text_content: String,
    pub text_font_size: f64,
    pub text_font_index: usize,
}

impl Default for AddPrimitiveParams {
    fn default() -> Self {
        Self {
            line_length: 100.0,
            line_angle_deg: 0.0,
            rect_width: 100.0,
            rect_height: 100.0,
            ellipse_rx: 60.0,
            ellipse_ry: 40.0,
            text_content: "Text".into(),
            text_font_size: 24.0,
            text_font_index: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GeneratorParams {
    pub spirograph: spirograph::SpirographParams,
    pub l_system: l_system::LSystemParams,
    pub voronoi: voronoi::VoronoiParams,
    pub wave: wave::WaveParams,
    pub grid: grid::GridParams,
    pub honeycomb: honeycomb::HoneycombParams,
    pub spiral: spiral::SpiralParams,
    pub resolution: u32,
    pub active_generator: GeneratorKind,
    pub add_primitive: AddPrimitiveParams,
}

impl Default for GeneratorParams {
    fn default() -> Self {
        Self {
            spirograph: Default::default(),
            l_system: Default::default(),
            voronoi: Default::default(),
            wave: Default::default(),
            grid: Default::default(),
            honeycomb: Default::default(),
            spiral: Default::default(),
            resolution: 200,
            add_primitive: Default::default(),
            active_generator: GeneratorKind::Spirograph,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeneratorKind {
    Spirograph,
    LSystem,
    Voronoi,
    Wave,
    Grid,
    Honeycomb,
    Spiral,
}

#[allow(dead_code)]
pub trait Generator {
    fn generate(&self, params: &GeneratorParams) -> Vec<PlotPath>;
    fn kind(&self) -> GeneratorKind;
    fn name(&self) -> &'static str;
}

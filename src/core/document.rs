use kurbo::Rect;
use serde::{Deserialize, Serialize};

use super::{Layer, Unit, CM_TO_UNITS};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    pub layers: Vec<Layer>,
    pub active_layer_idx: usize,
    pub paper_width_cm: f64,
    pub paper_height_cm: f64,
    pub snap_to_grid: bool,
    pub grid_size: f64,
    pub show_border: bool,
    pub border_dashed: bool,
    pub unit: Unit,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            layers: vec![Layer::new("Layer 1", Default::default())],
            active_layer_idx: 0,
            paper_width_cm: 21.0,
            paper_height_cm: 29.7,
            snap_to_grid: false,
            grid_size: 5.0,
            show_border: true,
            border_dashed: true,
            unit: Unit::Cm,
        }
    }
}

impl Document {
    pub fn canvas_bounds(&self) -> Rect {
        let w = self.paper_width_cm * CM_TO_UNITS;
        let h = self.paper_height_cm * CM_TO_UNITS;
        Rect::from_origin_size(kurbo::Point::new(-w / 2.0, -h / 2.0), kurbo::Size::new(w, h))
    }

    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.layers.get_mut(self.active_layer_idx)
    }

    pub fn active_layer(&self) -> Option<&Layer> {
        self.layers.get(self.active_layer_idx)
    }

    #[allow(dead_code)]
    pub fn snap_value(&self, v: f64) -> f64 {
        if self.snap_to_grid && self.grid_size > 0.0 {
            (v / self.grid_size).round() * self.grid_size
        } else {
            v
        }
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn remove_layer(&mut self, idx: usize) -> bool {
        if idx < self.layers.len() && self.layers.len() > 1 {
            self.layers.remove(idx);
            self.active_layer_idx = self.active_layer_idx.min(self.layers.len() - 1);
            true
        } else {
            false
        }
    }
}

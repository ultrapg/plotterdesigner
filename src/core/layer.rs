use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::PlotPath;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pen {
    pub color: Color32,
    pub width_mm: f64,
    pub lift_height: f64,
    pub label: String,
}

impl Default for Pen {
    fn default() -> Self {
        Self {
            color: Color32::BLACK,
            width_mm: 0.5,
            lift_height: 5.0,
            label: "Black".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layer {
    pub id: Uuid,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub pen: Pen,
    pub paths: Vec<PlotPath>,
}

impl Layer {
    pub fn new(name: impl Into<String>, pen: Pen) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            visible: true,
            locked: false,
            pen,
            paths: Vec::new(),
        }
    }
}

pub mod path;
pub mod layer;
pub mod document;
pub mod text;

pub use path::PlotPath;
pub use layer::{Layer, Pen};
pub use document::Document;

/// Conversion factor from cm to internal world units.
/// Matches 3.7795 px/mm used for pen preview (37.795 px/cm).
pub const CM_TO_UNITS: f64 = 37.795;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Unit {
    Cm,
    Mm,
}

impl Unit {
    pub fn label(&self) -> &'static str {
        match self {
            Unit::Cm => "cm",
            Unit::Mm => "mm",
        }
    }

    pub fn factor(&self) -> f64 {
        match self {
            Unit::Cm => 1.0,
            Unit::Mm => 10.0,
        }
    }
}

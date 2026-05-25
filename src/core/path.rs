use kurbo::BezPath;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlotPath {
    pub id: Uuid,
    pub name: String,
    pub geometry: BezPath,
    pub closed: bool,
    pub visible: bool,
    pub filled: bool,
}

impl PlotPath {
    pub fn new(geometry: BezPath, closed: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            geometry,
            closed,
            visible: true,
            filled: false,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

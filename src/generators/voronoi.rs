use kurbo::{BezPath, PathEl, Point};
use rand::SeedableRng;
use voronoice::{BoundingBox, Point as VPoint, VoronoiBuilder};

use super::{Generator, GeneratorKind, GeneratorParams};
use crate::core::PlotPath;

#[derive(Clone, Debug)]
pub struct VoronoiParams {
    pub num_points: u32,
    pub width: f64,
    pub height: f64,
    pub seed: u64,
}

impl Default for VoronoiParams {
    fn default() -> Self {
        Self {
            num_points: 50,
            width: 400.0,
            height: 400.0,
            seed: 42,
        }
    }
}

pub struct VoronoiGenerator;

impl Generator for VoronoiGenerator {
    fn generate(&self, params: &GeneratorParams) -> Vec<PlotPath> {
        let p = &params.voronoi;

        // Generate random sites
        use rand::Rng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(p.seed);
        let half_w = p.width / 2.0;
        let half_h = p.height / 2.0;
        let sites: Vec<VPoint> = (0..p.num_points)
            .map(|_| VPoint {
                x: rng.random_range(-half_w..half_w),
                y: rng.random_range(-half_h..half_h),
            })
            .collect();

        let diagram = VoronoiBuilder::default()
            .set_sites(sites)
            .set_bounding_box(BoundingBox::new_centered(p.width, p.height))
            .build();

        let diagram = match diagram {
            Some(d) => d,
            None => return vec![],
        };

        let mut paths = Vec::new();
        for cell in diagram.iter_cells() {
            let mut bez = BezPath::new();
            let mut first = true;
            for v in cell.iter_vertices() {
                let pt = Point::new(v.x, v.y);
                if first {
                    bez.push(PathEl::MoveTo(pt));
                    first = false;
                } else {
                    bez.push(PathEl::LineTo(pt));
                }
            }
            bez.push(PathEl::ClosePath);
            paths.push(PlotPath::new(bez, true));
        }

        paths
    }

    fn kind(&self) -> GeneratorKind {
        GeneratorKind::Voronoi
    }

    fn name(&self) -> &'static str {
        "Voronoi"
    }
}

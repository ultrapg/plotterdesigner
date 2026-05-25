pub mod svg_writer;
pub mod optimizer;

pub use svg_writer::SvgWriter;

/// Options for SVG export, including experimental features.
#[derive(Clone, Debug)]
pub struct ExportOptions {
    /// Experimental: remove overlapping/redundant line segments.
    /// When enabled, the exporter will detect and remove segments that
    /// are nearly coincident with other segments, reducing pen travel.
    pub remove_overlaps: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            remove_overlaps: false,
        }
    }
}

/// Remove overlapping segments from a collection of optimized paths.
/// Two segments are considered overlapping if they are nearly coincident
/// (within `tolerance` world units) AND oriented in the same direction,
/// OR if one segment is fully contained within another.
///
/// This is an EXPERIMENTAL feature and may not catch all cases.
fn remove_overlapping_segments(
    paths: &mut Vec<optimizer::OptimizedPath>,
    _tolerance: f64,
) {
    // Group paths by layer name
    let mut groups: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    for (i, p) in paths.iter().enumerate() {
        groups.entry(p.layer_name.clone()).or_default().push(i);
    }

    // Process each layer group independently
    let mut to_remove = vec![false; paths.len()];
    for (_layer_name, indices) in &groups {
        let mut segments: Vec<(usize, usize, kurbo::Point, kurbo::Point)> = Vec::new();
        for &idx in indices {
            let path = &paths[idx];
            let mut prev: Option<kurbo::Point> = None;
            let mut first = true;
            for el in path.path.elements() {
                let pt = match el {
                    kurbo::PathEl::MoveTo(p) => *p,
                    kurbo::PathEl::LineTo(p) => *p,
                    kurbo::PathEl::QuadTo(_, p) => *p,
                    kurbo::PathEl::CurveTo(_, _, p) => *p,
                    kurbo::PathEl::ClosePath => continue,
                };
                if let Some(a) = prev {
                    if !first {
                        segments.push((idx, segments.len(), a, pt));
                    }
                }
                if matches!(el, kurbo::PathEl::MoveTo(_)) {
                    first = true;
                }
                prev = Some(pt);
            }
        }

        // Compare each pair of segments for overlap
        for i in 0..segments.len() {
            if to_remove[segments[i].0] { continue; }
            for j in (i + 1)..segments.len() {
                if to_remove[segments[j].0] { continue; }
                if segments_are_coincident(segments[i].2, segments[i].3, segments[j].2, segments[j].3, _tolerance) {
                    to_remove[segments[j].0] = true;
                }
            }
        }
    }

    // Remove marked paths (in reverse order to preserve indices)
    for i in (0..paths.len()).rev() {
        if to_remove[i] {
            paths.remove(i);
        }
    }
}

/// Check if two line segments are nearly coincident (same line, overlapping).
fn segments_are_coincident(
    a1: kurbo::Point, a2: kurbo::Point,
    b1: kurbo::Point, b2: kurbo::Point,
    tolerance: f64,
) -> bool {
    // Check if endpoints are close (within tolerance)
    let d1 = a1.distance(b1);
    let d2 = a2.distance(b2);
    let d3 = a1.distance(b2);
    let d4 = a2.distance(b1);

    // Same direction: a1≈b1 and a2≈b2, or a1≈b2 and a2≈b1 (reversed)
    (d1 < tolerance && d2 < tolerance) || (d3 < tolerance && d4 < tolerance)
}

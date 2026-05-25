use kurbo::{BezPath, PathEl};

use super::optimizer::PathOptimizer;
use super::{remove_overlapping_segments, ExportOptions};
use crate::core::Document;

pub struct SvgWriter;

impl SvgWriter {
    #[allow(dead_code)]
    pub fn write_to_string(document: &Document) -> String {
        Self::write_with_options(document, &ExportOptions::default())
    }

    pub fn write_with_options(document: &Document, options: &ExportOptions) -> String {
        let mut optimized = PathOptimizer::optimize(document);

        if options.remove_overlaps {
            remove_overlapping_segments(&mut optimized, 5.0);
        }

        let bounds = document.canvas_bounds();

        let mut svg = String::new();
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="{0} {1} {2} {3}" width="{2}" height="{3}">
"#,
            bounds.x0 as i32,
            bounds.y0 as i32,
            bounds.width() as i32,
            bounds.height() as i32,
        ));

        let mut current_layer = String::new();
        for opt in &optimized {
            if opt.layer_name != current_layer {
                current_layer = opt.layer_name.clone();
                svg.push_str(&format!(
                    r#"  <g id="{}" stroke="{}" stroke-width="{}mm" fill="none">
"#,
                    opt.layer_name, opt.stroke_color, opt.stroke_width
                ));
            }

            let d = Self::bezpath_to_svg_d(&opt.path);
            svg.push_str(&format!("    <path d=\"{}\" />\n", d));
        }

        svg.push_str("</svg>\n");
        svg
    }

    fn bezpath_to_svg_d(path: &BezPath) -> String {
        let mut d = String::new();

        for el in path.elements() {
            match el {
                PathEl::MoveTo(p) => {
                    d.push_str(&format!("M {:.3} {:.3} ", p.x, p.y));
                }
                PathEl::LineTo(p) => {
                    d.push_str(&format!("L {:.3} {:.3} ", p.x, p.y));
                }
                PathEl::QuadTo(p1, p2) => {
                    d.push_str(&format!(
                        "Q {:.3} {:.3} {:.3} {:.3} ",
                        p1.x, p1.y, p2.x, p2.y
                    ));
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    d.push_str(&format!(
                        "C {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} ",
                        p1.x, p1.y, p2.x, p2.y, p3.x, p3.y
                    ));
                }
                PathEl::ClosePath => {
                    d.push('Z');
                }
            }
        }

        d
    }
}

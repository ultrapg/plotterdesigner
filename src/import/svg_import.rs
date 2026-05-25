use kurbo::{BezPath, PathEl, Point};
use usvg::tiny_skia_path::PathSegment;

use crate::core::{Document, Layer, Pen, PlotPath};

pub struct SvgImporter;

impl SvgImporter {
    pub fn import_from_str(svg_str: &str) -> Result<Document, String> {
        let tree = usvg::Tree::from_str(svg_str, &usvg::Options::default())
            .map_err(|e| format!("SVG parse error: {}", e))?;

        let mut document = Document::default();
        document.layers.clear();

        let mut current_layer = Layer::new("Imported", Pen::default());

        Self::extract_paths(tree.root(), &mut current_layer);

        if !current_layer.paths.is_empty() {
            document.layers.push(current_layer);
        }

        if document.layers.is_empty() {
            document.layers.push(Layer::new("Layer 1", Pen::default()));
        }
        document.active_layer_idx = 0;

        Ok(document)
    }

    fn extract_paths(node: &usvg::Group, layer: &mut Layer) {
        for child in node.children() {
            match child {
                usvg::Node::Path(path_data) => {
                    if let Some(bezpath) = Self::usvg_path_to_bezpath(path_data) {
                        let color = path_data
                            .fill()
                            .as_ref()
                            .map(|f| match f.paint() {
                                usvg::Paint::Color(c) => eframe::egui::Color32::from_rgb(
                                    c.red, c.green, c.blue,
                                ),
                                _ => eframe::egui::Color32::BLACK,
                            })
                            .unwrap_or(eframe::egui::Color32::BLACK);

                        let stroke_width = path_data
                            .stroke()
                            .as_ref()
                            .map(|s| s.width().get() as f64)
                            .unwrap_or(0.5);

                        let mut plot_path =
                            PlotPath::new(bezpath, false);
                        plot_path.name = format!("Path {}", layer.paths.len() + 1);

                        if layer.paths.is_empty() {
                            layer.pen.color = color;
                            layer.pen.width_mm = stroke_width;
                        }

                        layer.paths.push(plot_path);
                    }
                }
                usvg::Node::Group(g) => {
                    Self::extract_paths(g, layer);
                }
                _ => {}
            }
        }
    }

    fn usvg_path_to_bezpath(path: &usvg::Path) -> Option<BezPath> {
        let ts_path = path.data();
        let mut bez = BezPath::new();

        for seg in ts_path.segments() {
            match seg {
                PathSegment::MoveTo(p) => {
                    bez.push(PathEl::MoveTo(Point::new(p.x as f64, p.y as f64)));
                }
                PathSegment::LineTo(p) => {
                    bez.push(PathEl::LineTo(Point::new(p.x as f64, p.y as f64)));
                }
                PathSegment::QuadTo(c, p) => {
                    bez.push(PathEl::QuadTo(
                        Point::new(c.x as f64, c.y as f64),
                        Point::new(p.x as f64, p.y as f64),
                    ));
                }
                PathSegment::CubicTo(c1, c2, p) => {
                    bez.push(PathEl::CurveTo(
                        Point::new(c1.x as f64, c1.y as f64),
                        Point::new(c2.x as f64, c2.y as f64),
                        Point::new(p.x as f64, p.y as f64),
                    ));
                }
                PathSegment::Close => {
                    bez.push(PathEl::ClosePath);
                }
            }
        }

        if bez.elements().is_empty() {
            None
        } else {
            Some(bez)
        }
    }
}

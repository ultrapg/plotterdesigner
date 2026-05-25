use kurbo::{BezPath, PathEl, Point};
use ttf_parser::{Face, OutlineBuilder};

pub struct FontFace {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub const FONTS: &[FontFace] = &[
    FontFace { name: "Sans", data: include_bytes!("../../fonts/DejaVuSans.ttf") },
    FontFace { name: "Sans Bold", data: include_bytes!("../../fonts/DejaVuSans-Bold.ttf") },
];

/// Convert a string to BezPaths representing the text outline.
/// font_index selects which font from FONTS to use.
/// Returns paths centered at origin.
pub fn text_to_paths(text: &str, font_size: f64, font_index: usize) -> Vec<BezPath> {
    let font_data = if font_index < FONTS.len() {
        FONTS[font_index].data
    } else {
        FONTS[0].data
    };

    let face = match Face::parse(font_data, 0) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let units_per_em = face.units_per_em() as f64;
    let scale = font_size / units_per_em;

    let total_width: f64 = text
        .chars()
        .filter_map(|ch| {
            let gid = face.glyph_index(ch).unwrap_or_default();
            face.glyph_hor_advance(gid).map(|a| a as f64 * scale)
        })
        .sum();

    let start_x = -total_width / 2.0;
    let mut paths = Vec::new();
    let mut x_cursor = start_x;

    for ch in text.chars() {
        let glyph_id = face
            .glyph_index(ch)
            .unwrap_or_else(|| face.glyph_index(' ').unwrap_or_default());
        let mut builder = BezOutlineBuilder::new(x_cursor, scale);

        if face.outline_glyph(glyph_id, &mut builder).is_some() {
            paths.extend(builder.take_paths());
        }

        if let Some(metrics) = face.glyph_hor_advance(glyph_id) {
            x_cursor += metrics as f64 * scale;
        }
    }

    paths
}

struct BezOutlineBuilder {
    x_cursor: f64,
    scale: f64,
    current_path: Option<BezPath>,
    paths: Vec<BezPath>,
    first_point: Option<Point>,
}

impl BezOutlineBuilder {
    fn new(x_cursor: f64, scale: f64) -> Self {
        Self {
            x_cursor,
            scale,
            current_path: None,
            paths: Vec::new(),
            first_point: None,
        }
    }

    fn transform(&self, x: f32, y: f32) -> Point {
        Point::new(
            self.x_cursor + x as f64 * self.scale,
            -(y as f64) * self.scale,
        )
    }

    fn take_paths(&mut self) -> Vec<BezPath> {
        if let Some(path) = self.current_path.take() {
            self.paths.push(path);
        }
        self.paths.drain(..).collect()
    }
}

impl OutlineBuilder for BezOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.first_point.is_some() {
            if let Some(ref mut path) = self.current_path {
                path.push(PathEl::ClosePath);
            }
        }
        let pt = self.transform(x, y);
        let path = self.current_path.get_or_insert_with(BezPath::new);
        path.push(PathEl::MoveTo(pt));
        self.first_point = Some(pt);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let pt = self.transform(x, y);
        if let Some(ref mut path) = self.current_path {
            path.push(PathEl::LineTo(pt));
        }
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let cp = self.transform(cx, cy);
        let pt = self.transform(x, y);
        if let Some(ref mut path) = self.current_path {
            path.push(PathEl::QuadTo(cp, pt));
        }
    }

    fn curve_to(&mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32) {
        let cp1 = self.transform(cx1, cy1);
        let cp2 = self.transform(cx2, cy2);
        let pt = self.transform(x, y);
        if let Some(ref mut path) = self.current_path {
            path.push(PathEl::CurveTo(cp1, cp2, pt));
        }
    }

    fn close(&mut self) {
        if let Some(ref mut path) = self.current_path {
            path.push(PathEl::ClosePath);
        }
        self.first_point = None;
    }
}

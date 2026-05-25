use eframe::egui::{self, DragValue, Ui};
use kurbo::{BezPath, PathEl, Point};

use crate::core::text::{text_to_paths, FONTS};
use crate::core::{Document, PlotPath};
use crate::generators::{
    grid::GridGenerator, honeycomb::HoneycombGenerator, l_system::LSystemGenerator,
    spirograph::SpirographGenerator, spiral::SpiralGenerator, voronoi::VoronoiGenerator,
    wave::WaveGenerator, Generator, GeneratorKind, GeneratorParams,
};

pub struct GeneratorPanel;

impl GeneratorPanel {
    pub fn ui(ui: &mut Ui, document: &mut Document, params: &mut GeneratorParams) {
        ui.heading("Create");

        // ── Collapsible: Generators ──
        ui.collapsing("Generators", |ui| {
            ui.horizontal(|ui| {
                ui.label("Type:").on_hover_text("Choose a procedural generator algorithm");
                egui::ComboBox::from_id_salt("gen_type")
                    .selected_text(format!("{:?}", params.active_generator))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut params.active_generator, GeneratorKind::Spirograph, "Spirograph");
                        ui.selectable_value(&mut params.active_generator, GeneratorKind::LSystem, "L-System");
                        ui.selectable_value(&mut params.active_generator, GeneratorKind::Voronoi, "Voronoi");
                        ui.selectable_value(&mut params.active_generator, GeneratorKind::Wave, "Wave");
                        ui.selectable_value(&mut params.active_generator, GeneratorKind::Grid, "Grid");
                        ui.selectable_value(&mut params.active_generator, GeneratorKind::Honeycomb, "Honeycomb");
                        ui.selectable_value(&mut params.active_generator, GeneratorKind::Spiral, "Spiral");
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Resolution:");
                if ui.add(egui::DragValue::new(&mut params.resolution).speed(1).range(10..=10000)).changed() { }
                ui.label("samples").on_hover_text("Number of sample points for curve generation (higher = smoother)");
            });

            ui.separator();

            match params.active_generator {
                GeneratorKind::Spirograph => spirograph_ui(ui, params, document),
                GeneratorKind::LSystem => lsystem_ui(ui, params, document),
                GeneratorKind::Voronoi => voronoi_ui(ui, params, document),
                GeneratorKind::Wave => wave_ui(ui, params, document),
                GeneratorKind::Grid => grid_ui(ui, params, document),
                GeneratorKind::Honeycomb => honeycomb_ui(ui, params, document),
                GeneratorKind::Spiral => spiral_ui(ui, params, document),
            }
        });

        ui.separator();

        // ── Add section: manual shapes with parameters ──
        ui.collapsing("Add", |ui| {
            add_section_ui(ui, document, params);
        });
    }
}

fn add_section_ui(ui: &mut Ui, document: &mut Document, params: &mut GeneratorParams) {
    let p = &mut params.add_primitive;

    ui.horizontal(|ui| {
        ui.label("Shape:");
        if ui.button("Line").clicked() {
            let rad = p.line_angle_deg.to_radians();
            let half = p.line_length / 2.0;
            let mut bez = BezPath::new();
            bez.push(PathEl::MoveTo(Point::new(-half * rad.cos(), -half * rad.sin())));
            bez.push(PathEl::LineTo(Point::new(half * rad.cos(), half * rad.sin())));
            add_path(document, bez, "Line");
        }
        if ui.button("Rect").clicked() {
            let hw = p.rect_width / 2.0;
            let hh = p.rect_height / 2.0;
            let mut bez = BezPath::new();
            bez.push(PathEl::MoveTo(Point::new(-hw, -hh)));
            bez.push(PathEl::LineTo(Point::new(hw, -hh)));
            bez.push(PathEl::LineTo(Point::new(hw, hh)));
            bez.push(PathEl::LineTo(Point::new(-hw, hh)));
            bez.push(PathEl::ClosePath);
            add_path(document, bez, "Rectangle");
        }
        if ui.button("Ellipse").clicked() {
            add_path(document, make_ellipse(p.ellipse_rx, p.ellipse_ry), "Ellipse");
        }
    });

    ui.separator();

    ui.label("Line:");
    ui.add(DragValue::new(&mut p.line_length).speed(1.0).prefix("Length: "))
        .on_hover_text("Line length in world units");
    ui.add(DragValue::new(&mut p.line_angle_deg).speed(1.0).prefix("Angle (°): "))
        .on_hover_text("Rotation angle in degrees");

    ui.separator();

    ui.label("Rectangle:");
    ui.add(DragValue::new(&mut p.rect_width).speed(1.0).prefix("Width: "))
        .on_hover_text("Rectangle width");
    ui.add(DragValue::new(&mut p.rect_height).speed(1.0).prefix("Height: "))
        .on_hover_text("Rectangle height");

    ui.separator();

    ui.label("Ellipse:");
    ui.add(DragValue::new(&mut p.ellipse_rx).speed(1.0).prefix("Rx: "))
        .on_hover_text("Ellipse horizontal radius");
    ui.add(DragValue::new(&mut p.ellipse_ry).speed(1.0).prefix("Ry: "))
        .on_hover_text("Ellipse vertical radius");

    ui.separator();

    ui.label("Text:");
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut p.text_content)
            .on_hover_text("Enter text to convert to path outlines");
        if ui.button("Add Text").clicked() && !p.text_content.is_empty() {
            let paths = text_to_paths(&p.text_content, p.text_font_size, p.text_font_index);
            if let Some(layer) = document.active_layer_mut() {
                for (i, bez) in paths.into_iter().enumerate() {
                    let name = if i == 0 {
                        format!("Text: {}", p.text_content)
                    } else {
                        format!("Text: {} (part {})", p.text_content, i + 1)
                    };
                    layer.paths.push(PlotPath::new(bez, true).with_name(name));
                }
            }
        }
    });
    let unit_label = document.unit.label();
    ui.horizontal(|ui| {
        ui.add(DragValue::new(&mut p.text_font_size).speed(1.0).prefix("Size: "))
            .on_hover_text(format!("Font size in {}", unit_label));
        let current_name = if p.text_font_index < FONTS.len() {
            FONTS[p.text_font_index].name
        } else {
            "?"
        };
        egui::ComboBox::from_id_salt("font_selector")
            .selected_text(current_name)
            .show_ui(ui, |ui| {
                for (i, face) in FONTS.iter().enumerate() {
                    ui.selectable_value(&mut p.text_font_index, i, face.name);
                }
            });
    });
}

fn add_path(document: &mut Document, bez: BezPath, name: &str) {
    if let Some(layer) = document.active_layer_mut() {
        layer.paths.push(PlotPath::new(bez, true).with_name(name));
    }
}

fn make_ellipse(rx: f64, ry: f64) -> BezPath {
    let mut bez = BezPath::new();
    let c = 0.551784;
    bez.push(PathEl::MoveTo(Point::new(0.0, -ry)));
    bez.push(PathEl::CurveTo(
        Point::new(rx * c, -ry),
        Point::new(rx, -ry * c),
        Point::new(rx, 0.0),
    ));
    bez.push(PathEl::CurveTo(
        Point::new(rx, ry * c),
        Point::new(rx * c, ry),
        Point::new(0.0, ry),
    ));
    bez.push(PathEl::CurveTo(
        Point::new(-rx * c, ry),
        Point::new(-rx, ry * c),
        Point::new(-rx, 0.0),
    ));
    bez.push(PathEl::CurveTo(
        Point::new(-rx, -ry * c),
        Point::new(-rx * c, -ry),
        Point::new(0.0, -ry),
    ));
    bez.push(PathEl::ClosePath);
    bez
}

// ── Generator UIs (with collapsible description per algorithm) ──

fn spirograph_ui(ui: &mut Ui, params: &mut GeneratorParams, document: &mut Document) {
    let p = &mut params.spirograph;
    ui.label("Spirograph").on_hover_text("Creates a spirograph curve from rotating radii");
    ui.add(DragValue::new(&mut p.outer_radius).speed(1.0).prefix("Outer R: "));
    ui.add(DragValue::new(&mut p.inner_radius).speed(1.0).prefix("Inner R: "));
    ui.add(DragValue::new(&mut p.pen_offset).speed(1.0).prefix("Pen offset: "));
    ui.add(DragValue::new(&mut p.revolutions).speed(0.1).prefix("Revolutions: "));
    ui.collapsing("Help", |ui| {
        ui.label("A spirograph traces a curve from a point on a smaller circle rolling inside a larger one. Repeating patterns emerge based on the ratio of radii.");
    });
    gen_buttons(ui, params, GeneratorKind::Spirograph, document);
}

fn lsystem_ui(ui: &mut Ui, params: &mut GeneratorParams, document: &mut Document) {
    let p = &mut params.l_system;
    ui.label("L-System").on_hover_text("Generates fractal patterns from string rewriting rules");
    ui.text_edit_singleline(&mut p.axiom)
        .on_hover_text("Starting string for the L-System. Characters are replaced each iteration per the rules below.");
    ui.add(DragValue::new(&mut p.iterations).speed(1).prefix("Iterations: "));
    let mut angle_deg = p.angle.to_degrees();
    if ui.add(DragValue::new(&mut angle_deg).speed(1.0).prefix("Angle (°): ")).changed() {
        p.angle = angle_deg.to_radians();
    }
    ui.add(DragValue::new(&mut p.segment_length).speed(1.0).prefix("Step: "));

    ui.collapsing("Help", |ui| {
        ui.label("L-Systems generate fractals by repeatedly replacing characters in a string with production rules. F=draw forward, f=move forward, +=turn right, -=turn left, [=push state, ]=pop state.");
    });
    ui.collapsing("Rules", |ui| {
        ui.label("One rule per line: Char=Replacement");
        ui.add_sized([ui.available_width(), 80.0], egui::TextEdit::multiline(&mut p.rules_text))
            .on_hover_text("Rewrite rules. Example: F=F[+F]F[-F]F produces a branching tree pattern.");
    });

    gen_buttons(ui, params, GeneratorKind::LSystem, document);
}

fn voronoi_ui(ui: &mut Ui, params: &mut GeneratorParams, document: &mut Document) {
    let p = &mut params.voronoi;
    ui.label("Voronoi").on_hover_text("Generates a Voronoi diagram from random seed points");
    ui.add(DragValue::new(&mut p.num_points).speed(1.0).prefix("Points: "));
    ui.add(DragValue::new(&mut p.width).speed(1.0).prefix("Width: "));
    ui.add(DragValue::new(&mut p.height).speed(1.0).prefix("Height: "));
    ui.collapsing("Help", |ui| {
        ui.label("A Voronoi diagram partitions the canvas into regions around seed points. Each region contains all points closer to its seed than any other.");
    });
    gen_buttons(ui, params, GeneratorKind::Voronoi, document);
}

fn wave_ui(ui: &mut Ui, params: &mut GeneratorParams, document: &mut Document) {
    let p = &mut params.wave;
    ui.label("Wave").on_hover_text("Generates sine wave patterns across the canvas");
    ui.add(DragValue::new(&mut p.amplitude).speed(1.0).prefix("Amplitude: "));
    ui.add(DragValue::new(&mut p.frequency).speed(0.5).prefix("Frequency: "));
    ui.add(DragValue::new(&mut p.num_waves).speed(1).prefix("Waves: "));
    ui.add(DragValue::new(&mut p.width).speed(1.0).prefix("Width: "));
    ui.collapsing("Help", |ui| {
        ui.label("Generates parallel sine waves. Resolution (samples per wave) is controlled by the Resolution setting above. Higher values give smoother curves.");
    });
    gen_buttons(ui, params, GeneratorKind::Wave, document);
}

fn grid_ui(ui: &mut Ui, params: &mut GeneratorParams, document: &mut Document) {
    let p = &mut params.grid;
    ui.label("Grid").on_hover_text("Generates a rectangular grid of lines");
    ui.add(DragValue::new(&mut p.cols).speed(1).prefix("Columns: "));
    ui.add(DragValue::new(&mut p.rows).speed(1).prefix("Rows: "));
    ui.add(DragValue::new(&mut p.width).speed(1.0).prefix("Width: "));
    ui.add(DragValue::new(&mut p.height).speed(1.0).prefix("Height: "));
    ui.collapsing("Help", |ui| {
        ui.label("Creates evenly spaced horizontal and vertical lines forming a rectangular grid.");
    });
    gen_buttons(ui, params, GeneratorKind::Grid, document);
}

fn honeycomb_ui(ui: &mut Ui, params: &mut GeneratorParams, document: &mut Document) {
    let p = &mut params.honeycomb;
    ui.label("Honeycomb").on_hover_text("Generates a honeycomb/hexagon tiling pattern");
    ui.add(DragValue::new(&mut p.cell_radius).speed(1.0).prefix("Radius: "));
    ui.add(DragValue::new(&mut p.cols).speed(1).prefix("Columns: "));
    ui.add(DragValue::new(&mut p.rows).speed(1).prefix("Rows: "));
    ui.collapsing("Help", |ui| {
        ui.label("Generates a hexagonal honeycomb tiling. Each cell is a regular hexagon. Odd rows have one fewer cell to maintain the staggered pattern.");
    });
    gen_buttons(ui, params, GeneratorKind::Honeycomb, document);
}

fn spiral_ui(ui: &mut Ui, params: &mut GeneratorParams, document: &mut Document) {
    let p = &mut params.spiral;
    ui.label("Spiral").on_hover_text("Generates an Archimedean spiral");
    ui.add(DragValue::new(&mut p.turns).speed(0.5).prefix("Turns: "));
    ui.add(DragValue::new(&mut p.max_radius).speed(1.0).prefix("Max Radius: "));
    ui.add(DragValue::new(&mut p.growth).speed(0.5).prefix("Growth: "));
    ui.collapsing("Help", |ui| {
        ui.label("Archimedean spiral: the radius increases linearly with angle. Growth controls how the radius scales (1.0 = linear, < 1 = slower outer growth, > 1 = faster outer growth).");
    });
    gen_buttons(ui, params, GeneratorKind::Spiral, document);
}

fn gen_buttons(ui: &mut Ui, params: &mut GeneratorParams, kind: GeneratorKind, document: &mut Document) {
    ui.horizontal(|ui| {
        if ui.button("Generate")
            .on_hover_text("Add a new path from this generator")
            .clicked()
        {
            let generator: Box<dyn Generator> = match kind {
                GeneratorKind::Spirograph => Box::new(SpirographGenerator),
                GeneratorKind::LSystem => Box::new(LSystemGenerator),
                GeneratorKind::Voronoi => Box::new(VoronoiGenerator),
                GeneratorKind::Wave => Box::new(WaveGenerator),
                GeneratorKind::Grid => Box::new(GridGenerator),
                GeneratorKind::Honeycomb => Box::new(HoneycombGenerator),
                GeneratorKind::Spiral => Box::new(SpiralGenerator),
            };
            if let Some(layer) = document.active_layer_mut() {
                let paths = generator.generate(params);
                for path in paths {
                    layer.paths.push(path);
                }
            }
        }
        if ui.button("Replace")
            .on_hover_text("Replace all paths in the active layer with this generator's output")
            .clicked()
        {
            let generator: Box<dyn Generator> = match kind {
                GeneratorKind::Spirograph => Box::new(SpirographGenerator),
                GeneratorKind::LSystem => Box::new(LSystemGenerator),
                GeneratorKind::Voronoi => Box::new(VoronoiGenerator),
                GeneratorKind::Wave => Box::new(WaveGenerator),
                GeneratorKind::Grid => Box::new(GridGenerator),
                GeneratorKind::Honeycomb => Box::new(HoneycombGenerator),
                GeneratorKind::Spiral => Box::new(SpiralGenerator),
            };
            if let Some(layer) = document.active_layer_mut() {
                layer.paths.clear();
                let paths = generator.generate(params);
                for path in paths {
                    layer.paths.push(path);
                }
            }
        }
    });
}

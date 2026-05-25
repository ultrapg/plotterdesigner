mod app;
mod canvas;
mod core;
mod export;
mod generators;
mod import;
mod manipulate;
mod ui;

use eframe::egui::ViewportBuilder;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Plotter Designer")
            .with_inner_size([1400.0, 900.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "Plotter Designer",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::default()))),
    )
}

# plotterdesigner

A real-time vector plotter designer built with Rust and egui. Create, manipulate, and export layered SVG files optimized for pen plotters.

## Features

- **Parametric generators** — Spirograph, L-System, Voronoi, Wave, Grid, Honeycomb, Spiral
- **Pen plotter optimization** — nearest-neighbor TSP path sorting minimizes pen travel
- **Layer management** — multiple layers with visibility toggles, reordering, per-layer pen settings
- **Interactive canvas** — pan/zoom, snap-to-grid, configurable paper border (cm/mm)
- **Path manipulation** — select, move, scale, rotate, point-edit individual vertices
- **Fill support** — toggle filled/outline rendering per path
- **Text-to-path** — convert text to vector outlines (bundled DejaVu Sans fonts)
- **SVG import/export** — import SVG files via usvg, export layered SVG with stroke widths
- **Export optimization** (experimental) — optional removal of overlapping/redundant paths
- **Keyboard shortcuts** — Delete/Backspace, arrow keys (×10 with Shift), Ctrl+D duplicate
- **Project save/open** — serialized `.pdp` format via RON

## Screenshots

*(Add screenshots here)*

## Usage

```
cargo run --release
```

### Controls

| Action | Input |
|---|---|
| Select tool | Left-click to select, drag to move |
| Multi-select | Shift + click |
| Drag-select | Click empty canvas and drag |
| Resize | Drag any of the 8 blue handles |
| Rotate | Drag the green circle above selection |
| Point-edit | Click and drag vertex handles |
| Pan | Right-click + drag, or middle-click + drag, or Shift + drag |
| Zoom | Scroll wheel |
| Delete | Delete or Backspace |
| Duplicate | Ctrl + D |
| Nudge | Arrow keys (×10 with Shift) |

## Build

### Prerequisites

- Rust 1.75+ (edition 2021)
- OpenGL 3.3+ (for Glow renderer)

### Build & Run

```bash
cargo build --release
cargo run --release
```

### Platform Notes

- **Raspberry Pi (V3D GPU)**: Uses Glow (OpenGL) renderer — WGPU is incompatible with V3D's 4-color-attachment limit
- **Linux**: Tested on aarch64 (Raspberry Pi 5) and x86_64
- **Wayland/X11**: Both supported

## Architecture

```
src/
├── main.rs              # Entry point, Glow renderer config
├── app.rs               # Main App, menu bar, keyboard shortcuts, dialog dispatch
├── canvas/              # Interactive canvas
│   ├── transform.rs     # ViewportTransform (pan/zoom/screen↔world)
│   ├── interaction.rs   # InteractionState, TransformMode, handle positions
│   └── renderer.rs      # Path rendering, hit testing, drag-select
├── core/                # Data model
│   ├── document.rs      # Document (layers, paper settings, unit)
│   ├── path.rs          # PlotPath (BezPath wrapper with metadata)
│   ├── layer.rs         # Layer, Pen
│   ├── text.rs          # text_to_paths via ttf-parser
│   └── mod.rs           # Unit enum, CM_TO_UNITS constant
├── export/              # SVG export
│   ├── svg_writer.rs    # Layered SVG generation
│   ├── optimizer.rs     # TSP path sorting, overlap removal
│   └── mod.rs           # ExportOptions
├── generators/          # Procedural path generators
│   ├── spirograph.rs
│   ├── l_system.rs
│   ├── voronoi.rs
│   ├── wave.rs
│   ├── grid.rs
│   ├── honeycomb.rs
│   └── spiral.rs
├── import/
│   └── svg_import.rs    # SVG → Document via usvg
├── manipulate/          # Path transformation helpers
│   └── mod.rs           # translate, scale, rotate, edit_point, duplicate
└── ui/
    ├── toolbar.rs        # Select/Pan tool selection
    ├── layer_panel.rs    # Layer tree, path list, transform controls
    ├── generator_panel.rs# Generator controls, primitives (Line/Rect/Ellipse/Text)
    ├── pen_preview.rs    # Stroke width preview toggle
    └── dialogs.rs        # Export/Import/Save/Open modals
```

### Key Design Decisions

- **Glow over WGPU**: WGPU requests 8 color attachments but Raspberry Pi V3D GPU only supports 4
- **kurbo::BezPath**: Primary path type (not lyon) — has curve math and SVG export built-in
- **RON serialization**: `.pdp` project files use Rusty Object Notation
- **Deferred-action pattern**: Renderer writes `PendingAction`, App applies at end of frame to avoid concurrent `&mut` borrow conflicts
- **Single source of truth for handles**: `compute_handle_screen_positions()` used by both hit testing and rendering

## License

GNU General Public License v3.0

use kurbo::{Affine, Point};

#[derive(Clone, Debug)]
pub struct ViewportTransform {
    pub pan_x: f64,
    pub pan_y: f64,
    pub zoom: f64,
}

impl Default for ViewportTransform {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }
}

impl ViewportTransform {
    pub fn affine(&self) -> Affine {
        Affine::translate((self.pan_x, self.pan_y)) * Affine::scale(self.zoom)
    }

    pub fn screen_to_world(&self, screen_x: f64, screen_y: f64) -> Point {
        Point::new(
            (screen_x - self.pan_x) / self.zoom,
            (screen_y - self.pan_y) / self.zoom,
        )
    }

    pub fn world_to_screen(&self, world_x: f64, world_y: f64) -> Point {
        Point::new(
            world_x * self.zoom + self.pan_x,
            world_y * self.zoom + self.pan_y,
        )
    }

    pub fn pan_by(&mut self, dx: f64, dy: f64) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    pub fn zoom_at(&mut self, center_x: f64, center_y: f64, factor: f64) {
        let world_before = self.screen_to_world(center_x, center_y);
        self.zoom = (self.zoom * factor).clamp(0.05, 50.0);
        let world_after = self.screen_to_world(center_x, center_y);
        self.pan_x += (world_after.x - world_before.x) * self.zoom;
        self.pan_y += (world_after.y - world_before.y) * self.zoom;
    }
}

//! Camera system for view control and world/screen coordinate conversion

use glam::{Vec2, vec2};

/// Rectangle in world coordinates
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Check if a point is inside the rectangle
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width &&
        y >= self.y && y <= self.y + self.height
    }
}

/// Camera system for 2D view control
///
/// Handles view transformations, zoom, and panning.
/// The camera uses a simple orthographic projection.
#[derive(Debug, Clone)]
pub struct Camera {
    /// World position the camera is centered on
    pub position: Vec2,

    /// Zoom level (pixels per meter)
    /// 1.0 = 1 pixel per meter
    /// 2.0 = 2 pixels per meter (zoomed in)
    /// 0.5 = 0.5 pixels per meter (zoomed out)
    pub zoom: f32,

    /// Camera rotation in radians (not used in Phase 1)
    pub rotation: f32,

    /// Viewport size in pixels (width, height)
    pub viewport: (u32, u32),
}

impl Camera {
    /// Create a new camera
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            position: vec2(0.0, 0.0),
            zoom: 1.0,
            rotation: 0.0,
            viewport: (viewport_width, viewport_height),
        }
    }

    /// Convert world coordinates to screen coordinates
    ///
    /// World coordinates: meters from origin
    /// Screen coordinates: pixels from top-left corner
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        // Offset from camera center
        let relative = world_pos - self.position;

        // Apply zoom
        let scaled = relative * self.zoom;

        // Convert to screen space (origin at center, then move to top-left)
        let screen_x = (self.viewport.0 as f32 / 2.0) + scaled.x;
        let screen_y = (self.viewport.1 as f32 / 2.0) - scaled.y; // Y is inverted (down = positive)

        vec2(screen_x, screen_y)
    }

    /// Convert screen coordinates to world coordinates
    ///
    /// Screen coordinates: pixels from top-left corner
    /// World coordinates: meters from origin
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        // Convert from top-left origin to center origin
        let centered_x = screen_pos.x - (self.viewport.0 as f32 / 2.0);
        let centered_y = (self.viewport.1 as f32 / 2.0) - screen_pos.y; // Y is inverted

        // Undo zoom
        let unscaled = vec2(centered_x, centered_y) / self.zoom;

        // Add camera position
        self.position + unscaled
    }

    /// Pan the camera by a screen delta (in pixels)
    pub fn pan(&mut self, delta: Vec2) {
        // Convert screen delta to world delta
        let world_delta = delta / self.zoom;
        self.position -= vec2(world_delta.x, -world_delta.y); // Y is inverted
    }

    /// Zoom at a specific screen position
    ///
    /// This keeps the point under the cursor stationary while zooming
    pub fn zoom_at(&mut self, screen_pos: Vec2, zoom_delta: f32) {
        // Get the world position under the cursor before zoom
        let world_before = self.screen_to_world(screen_pos);

        // Apply zoom (clamped to reasonable range)
        self.zoom = (self.zoom * zoom_delta).clamp(0.1, 100.0);

        // Get the world position under the cursor after zoom
        let world_after = self.screen_to_world(screen_pos);

        // Adjust camera position to keep the point stationary
        self.position += world_before - world_after;
    }

    /// Zoom in/out from the center of the screen
    pub fn zoom_center(&mut self, zoom_delta: f32) {
        let center = vec2(self.viewport.0 as f32 / 2.0, self.viewport.1 as f32 / 2.0);
        self.zoom_at(center, zoom_delta);
    }

    /// Get the visible world bounds for culling
    ///
    /// Returns a rectangle in world coordinates
    pub fn visible_bounds(&self) -> Rect {
        // Get the four corners of the screen in world coordinates
        let top_left = self.screen_to_world(vec2(0.0, 0.0));
        let bottom_right = self.screen_to_world(vec2(
            self.viewport.0 as f32,
            self.viewport.1 as f32,
        ));

        Rect::new(
            top_left.x,
            bottom_right.y,  // Y is inverted
            bottom_right.x - top_left.x,
            top_left.y - bottom_right.y,
        )
    }

    /// Update viewport size (called on window resize)
    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport = (width, height);
    }

    /// Center camera on a specific world position
    pub fn look_at(&mut self, world_pos: Vec2) {
        self.position = world_pos;
    }

    /// Get the world position at the center of the screen
    pub fn center(&self) -> Vec2 {
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_screen_at_origin() {
        let camera = Camera::new(800, 600);
        // World origin (0,0) should be screen center (400, 300)
        let screen = camera.world_to_screen(vec2(0.0, 0.0));
        assert_eq!(screen.x, 400.0);
        assert_eq!(screen.y, 300.0);
    }

    #[test]
    fn test_screen_to_world_at_center() {
        let camera = Camera::new(800, 600);
        // Screen center (400, 300) should be world origin (0, 0)
        let world = camera.screen_to_world(vec2(400.0, 300.0));
        assert!((world.x - 0.0).abs() < 0.001);
        assert!((world.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_zoom() {
        let mut camera = Camera::new(800, 600);
        camera.zoom = 2.0;

        // At 2x zoom, 1 meter = 2 pixels
        let screen = camera.world_to_screen(vec2(100.0, 0.0));
        assert_eq!(screen.x, 600.0); // 400 + (100 * 2)
        assert_eq!(screen.y, 300.0);
    }

    #[test]
    fn test_pan() {
        let mut camera = Camera::new(800, 600);
        camera.pan(vec2(100.0, 0.0)); // Pan right 100 pixels

        // Panning right moves camera position left
        assert_eq!(camera.position.x, -100.0);
    }

    #[test]
    fn test_visible_bounds() {
        let camera = Camera::new(800, 600);
        let bounds = camera.visible_bounds();

        // At 1.0 zoom, visible width = 800 meters, height = 600 meters
        assert_eq!(bounds.width, 800.0);
        assert_eq!(bounds.height, 600.0);

        // Centered at origin, so bounds should extend from -400 to +400 in x
        assert_eq!(bounds.x, -400.0);
        assert_eq!(bounds.y, -300.0);
    }
}

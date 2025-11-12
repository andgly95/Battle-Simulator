//! 2D pixel-based renderer using the `pixels` crate
//!
//! This is the Phase 1 renderer - simple, fast, and scalable to thousands of units.

use crate::camera::{Camera, Rect};
use anyhow::Result;
use glam::vec2;
use pixels::{Pixels, SurfaceTexture};
use winit::window::Window;

/// RGBA color (red, green, blue, alpha)
type Color = [u8; 4];

/// Color palette for rendering
pub mod colors {
    use super::Color;

    // Team colors
    pub const BLUE_ACTIVE: Color = [40, 120, 200, 255];     // Bright blue (Allied active)
    pub const BLUE_ROUTING: Color = [20, 60, 100, 255];     // Dark blue (Allied routing)
    pub const RED_ACTIVE: Color = [200, 40, 40, 255];       // Bright red (Enemy active)
    pub const RED_ROUTING: Color = [100, 20, 20, 255];      // Dark red (Enemy routing)

    // Terrain and UI
    pub const BACKGROUND: Color = [30, 35, 30, 255];        // Dark green background
    pub const GRID: Color = [50, 55, 50, 255];              // Slightly lighter grid
    pub const TEXT: Color = [220, 220, 220, 255];           // Light gray text

    // State indicators
    pub const WAVERING: Color = [200, 180, 40, 255];        // Yellow (low morale)
    pub const DESTROYED: Color = [80, 80, 80, 255];         // Gray (destroyed)
}

/// Simple 2D pixel-based renderer
pub struct PixelRenderer<'a> {
    /// Pixel buffer
    pixels: Pixels<'a>,

    /// Camera for view control
    camera: Camera,

    /// Viewport dimensions
    width: u32,
    height: u32,
}

impl<'a> PixelRenderer<'a> {
    /// Create a new pixel renderer
    pub fn new(window: &'a Window) -> Result<Self> {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window);

        let pixels = Pixels::new(window_size.width, window_size.height, surface_texture)?;

        // Start camera centered on origin, zoomed to show 1km x 1km area
        let mut camera = Camera::new(window_size.width, window_size.height);
        camera.zoom = window_size.width as f32 / 1000.0; // 1000 meters visible width
        camera.look_at(vec2(200.0, 350.0)); // Center on typical battle area

        Ok(Self {
            pixels,
            camera,
            width: window_size.width,
            height: window_size.height,
        })
    }

    /// Begin a new frame (clear the screen)
    pub fn begin_frame(&mut self) {
        let frame = self.pixels.frame_mut();
        
        // Fill with background color
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&colors::BACKGROUND);
        }
    }

    /// Draw a filled rectangle at screen coordinates
    fn draw_rect_screen(&mut self, x: i32, y: i32, width: u32, height: u32, color: Color) {
        let frame = self.pixels.frame_mut();

        for dy in 0..height as i32 {
            for dx in 0..width as i32 {
                let px = x + dx;
                let py = y + dy;

                // Bounds check
                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                    let index = ((py * self.width as i32 + px) * 4) as usize;
                    if index + 3 < frame.len() {
                        frame[index..index + 4].copy_from_slice(&color);
                    }
                }
            }
        }
    }

    /// Draw a single pixel at screen coordinates
    fn draw_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            let frame = self.pixels.frame_mut();
            let index = ((y * self.width as i32 + x) * 4) as usize;
            if index + 3 < frame.len() {
                frame[index..index + 4].copy_from_slice(&color);
            }
        }
    }

    /// Draw a line between two screen points (Bresenham's algorithm)
    fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color) {
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x0;
        let mut y = y0;

        loop {
            self.draw_pixel(x, y, color);

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Draw a grid for reference
    pub fn draw_grid(&mut self, grid_size: f32) {
        let visible = self.camera.visible_bounds();

        // Vertical lines
        let start_x = (visible.x / grid_size).floor() * grid_size;
        let mut x = start_x;
        while x <= visible.x + visible.width {
            let top = self.camera.world_to_screen(vec2(x, visible.y));
            let bottom = self.camera.world_to_screen(vec2(x, visible.y + visible.height));
            
            self.draw_line(
                top.x as i32, top.y as i32,
                bottom.x as i32, bottom.y as i32,
                colors::GRID,
            );
            
            x += grid_size;
        }

        // Horizontal lines
        let start_y = (visible.y / grid_size).floor() * grid_size;
        let mut y = start_y;
        while y <= visible.y + visible.height {
            let left = self.camera.world_to_screen(vec2(visible.x, y));
            let right = self.camera.world_to_screen(vec2(visible.x + visible.width, y));
            
            self.draw_line(
                left.x as i32, left.y as i32,
                right.x as i32, right.y as i32,
                colors::GRID,
            );
            
            y += grid_size;
        }
    }

    /// Draw a unit at world coordinates with world-space size
    /// size_meters: size in meters (e.g., 1.0 for a 1-meter soldier)
    pub fn draw_unit(&mut self, world_x: f32, world_y: f32, color: Color, size_meters: f32) {
        let screen_pos = self.camera.world_to_screen(vec2(world_x, world_y));

        // Convert world size to screen size based on zoom
        let screen_size = (size_meters * self.camera.zoom).max(1.0) as u32;

        // Center the rectangle on the position
        let x = screen_pos.x as i32 - (screen_size as i32 / 2);
        let y = screen_pos.y as i32 - (screen_size as i32 / 2);

        self.draw_rect_screen(x, y, screen_size, screen_size, color);
    }

    /// Draw a formation of individual soldiers
    ///
    /// Renders each soldier as a small pixel/square based on formation type
    /// Cohesion affects visual disorder - low cohesion = scattered, ragged formation
    pub fn draw_formation(
        &mut self,
        world_x: f32,
        world_y: f32,
        formation_type: &str,
        soldier_count: u32,
        cohesion: f32,
        color: Color,
    ) {
        // Calculate soldier size based on zoom (0.8 meters in world space)
        let soldier_size_meters = 0.8;

        // Disorder amount based on cohesion (lower cohesion = more scatter)
        let disorder = (1.0 - cohesion) * 4.0;

        // Simple pseudo-random offset generator (hash-based for consistency)
        let hash_offset = |i: u32| -> (f32, f32) {
            let h = i.wrapping_mul(2654435761); // Simple hash
            let x = ((h & 0xFFFF) as f32 / 65535.0 - 0.5) * disorder;
            let y = (((h >> 16) & 0xFFFF) as f32 / 65535.0 - 0.5) * disorder;
            (x, y)
        };

        match formation_type {
            "Line" => {
                // Line formation: soldiers in horizontal line
                // ~2 meters per soldier (close order)
                let spacing = 2.0;
                let total_width = soldier_count as f32 * spacing;
                let start_x = world_x - total_width / 2.0;

                for i in 0..soldier_count {
                    let base_x = start_x + i as f32 * spacing;
                    let (dx, dy) = hash_offset(i);
                    self.draw_unit(base_x + dx, world_y + dy, color, soldier_size_meters);
                }
            }

            "Column" => {
                // Column: 20 men wide, multiple ranks deep
                let width = 20;
                let ranks = (soldier_count as f32 / width as f32).ceil() as u32;
                let spacing = 1.5;

                let start_x = world_x - (width as f32 * spacing) / 2.0;
                let start_y = world_y - (ranks as f32 * spacing) / 2.0;

                let mut soldier_index = 0u32;
                for rank in 0..ranks {
                    let men_in_rank = if rank == ranks - 1 {
                        soldier_count - (rank * width)
                    } else {
                        width.min(soldier_count - rank * width)
                    };

                    for file in 0..men_in_rank {
                        let base_x = start_x + file as f32 * spacing;
                        let base_y = start_y + rank as f32 * spacing;
                        let (dx, dy) = hash_offset(soldier_index);
                        self.draw_unit(base_x + dx, base_y + dy, color, soldier_size_meters);
                        soldier_index += 1;
                    }
                }
            }

            "Square" => {
                // Hollow square for defense
                let side_length = (soldier_count as f32 / 4.0).sqrt().ceil() as u32;
                let spacing = 2.0;
                let half_size = side_length as f32 * spacing / 2.0;

                for i in 0..side_length {
                    let offset = i as f32 * spacing - half_size;
                    // Top, bottom, left, right sides
                    self.draw_unit(world_x + offset, world_y - half_size, color, soldier_size_meters);
                    self.draw_unit(world_x + offset, world_y + half_size, color, soldier_size_meters);
                    self.draw_unit(world_x - half_size, world_y + offset, color, soldier_size_meters);
                    self.draw_unit(world_x + half_size, world_y + offset, color, soldier_size_meters);
                }
            }

            "Skirmish" => {
                // Loose dispersed order
                let spacing = 4.0;
                let width = (soldier_count as f32).sqrt().ceil() as u32;
                let start_x = world_x - (width as f32 * spacing) / 2.0;
                let start_y = world_y - (width as f32 * spacing) / 2.0;

                for i in 0..soldier_count {
                    let row = i / width;
                    let col = i % width;
                    let x = start_x + col as f32 * spacing;
                    let y = start_y + row as f32 * spacing;
                    self.draw_unit(x, y, color, soldier_size_meters);
                }
            }

            _ => {
                // Fallback
                self.draw_unit(world_x, world_y, color, soldier_size_meters);
            }
        }
    }

    /// Draw terrain grid - fills entire background with terrain
    pub fn draw_terrain(&mut self, terrain: &battle_sim_core::terrain::Terrain) {
        let frame = self.pixels.frame_mut();

        // Iterate through every screen pixel and sample terrain
        for screen_y in 0..self.height {
            for screen_x in 0..self.width {
                // Convert screen pixel to world position
                let world_pos = self.camera.screen_to_world(vec2(screen_x as f32, screen_y as f32));

                // Get terrain type and elevation at this position
                let terrain_type = terrain.get_terrain_type(world_pos.x, world_pos.y);
                let elevation = terrain.get_elevation(world_pos.x, world_pos.y);

                // Base color from terrain type
                let mut color = terrain_type.color();

                // Shade based on elevation (darker = lower, lighter = higher)
                let elevation_shade = (elevation / 30.0).clamp(-0.3, 0.3);
                for i in 0..3 {
                    if elevation_shade > 0.0 {
                        color[i] = color[i].saturating_add((elevation_shade * 100.0) as u8);
                    } else {
                        color[i] = color[i].saturating_sub((-elevation_shade * 100.0) as u8);
                    }
                }

                // Set pixel color
                let index = ((screen_y * self.width + screen_x) * 4) as usize;
                if index + 3 < frame.len() {
                    frame[index..index + 4].copy_from_slice(&color);
                }
            }
        }
    }

    /// Complete frame and present to screen
    pub fn end_frame(&mut self) -> Result<()> {
        self.pixels.render()?;
        Ok(())
    }

    /// Handle window resize
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.pixels.resize_surface(width, height).unwrap();
        self.pixels.resize_buffer(width, height).unwrap();
        self.camera.resize(width, height);
    }

    /// Get mutable reference to camera
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Get immutable reference to camera
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get visible world bounds (for culling)
    pub fn visible_bounds(&self) -> Rect {
        self.camera.visible_bounds()
    }
}

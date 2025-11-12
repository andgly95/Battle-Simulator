//! Terrain system with elevation and terrain types
//!
//! Provides:
//! - Heightmap for elevation
//! - Terrain types (grass, forest, hill, marsh, road, etc.)
//! - Line of sight calculations
//! - Movement speed modifiers
//! - Combat effectiveness modifiers

use glam::Vec2;

/// Terrain type affecting movement and combat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainType {
    /// Open grass field - normal movement, no cover
    Grass,
    /// Forest - slow movement, provides cover, blocks line of sight
    Forest,
    /// Marsh/swamp - very slow movement, no cover
    Marsh,
    /// Road - fast movement, no cover
    Road,
    /// Hill/rocky - slower movement, high ground advantage
    Hill,
    /// Building/structure - provides cover, blocks line of sight
    Building,
    /// Water - impassable for infantry
    Water,
}

impl TerrainType {
    /// Movement speed multiplier (1.0 = normal)
    pub fn movement_speed_modifier(&self) -> f32 {
        match self {
            TerrainType::Grass => 1.0,
            TerrainType::Forest => 0.5,      // 50% speed in forest
            TerrainType::Marsh => 0.3,       // 30% speed in marsh
            TerrainType::Road => 1.5,        // 50% faster on roads
            TerrainType::Hill => 0.7,        // 30% slower uphill
            TerrainType::Building => 0.4,    // Slow in buildings
            TerrainType::Water => 0.0,       // Impassable
        }
    }

    /// Does this terrain block line of sight?
    pub fn blocks_line_of_sight(&self) -> bool {
        matches!(self, TerrainType::Forest | TerrainType::Building)
    }

    /// Does this terrain provide cover (reduces incoming fire effectiveness)?
    pub fn provides_cover(&self) -> bool {
        matches!(self, TerrainType::Forest | TerrainType::Building)
    }

    /// Cover effectiveness (0.0-1.0, how much it reduces incoming damage)
    pub fn cover_effectiveness(&self) -> f32 {
        match self {
            TerrainType::Forest => 0.5,      // 50% damage reduction
            TerrainType::Building => 0.7,    // 70% damage reduction
            _ => 0.0,
        }
    }

    /// Visual color for rendering [R, G, B, A]
    pub fn color(&self) -> [u8; 4] {
        match self {
            TerrainType::Grass => [80, 140, 60, 255],      // Green
            TerrainType::Forest => [40, 80, 30, 255],      // Dark green
            TerrainType::Marsh => [100, 90, 50, 255],      // Muddy brown
            TerrainType::Road => [120, 110, 100, 255],     // Gray-brown
            TerrainType::Hill => [130, 120, 100, 255],     // Rocky brown
            TerrainType::Building => [100, 100, 110, 255], // Gray
            TerrainType::Water => [60, 100, 180, 255],     // Blue
        }
    }
}

/// Terrain heightmap and type grid
#[derive(bevy_ecs::system::Resource)]
pub struct Terrain {
    /// Width of terrain in world units
    width: f32,
    /// Height of terrain in world units
    height: f32,
    /// Grid resolution (meters per cell)
    cell_size: f32,
    /// Elevation data (in meters)
    heightmap: Vec<f32>,
    /// Terrain type grid
    terrain_types: Vec<TerrainType>,
    /// Grid dimensions
    grid_width: usize,
    grid_height: usize,
}

impl Terrain {
    /// Create new flat terrain
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        let grid_width = (width / cell_size).ceil() as usize;
        let grid_height = (height / cell_size).ceil() as usize;
        let grid_size = grid_width * grid_height;

        Self {
            width,
            height,
            cell_size,
            heightmap: vec![0.0; grid_size],
            terrain_types: vec![TerrainType::Grass; grid_size],
            grid_width,
            grid_height,
        }
    }

    /// Get terrain dimensions
    pub fn dimensions(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    /// Get cell size
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Get grid dimensions
    pub fn grid_dimensions(&self) -> (usize, usize) {
        (self.grid_width, self.grid_height)
    }

    /// Convert world position to grid coordinates
    fn world_to_grid(&self, world_x: f32, world_y: f32) -> Option<(usize, usize)> {
        if world_x < 0.0 || world_y < 0.0 || world_x >= self.width || world_y >= self.height {
            return None;
        }

        let grid_x = (world_x / self.cell_size).floor() as usize;
        let grid_y = (world_y / self.cell_size).floor() as usize;

        if grid_x >= self.grid_width || grid_y >= self.grid_height {
            return None;
        }

        Some((grid_x, grid_y))
    }

    /// Get grid index from coordinates
    fn grid_index(&self, grid_x: usize, grid_y: usize) -> Option<usize> {
        if grid_x >= self.grid_width || grid_y >= self.grid_height {
            return None;
        }
        Some(grid_y * self.grid_width + grid_x)
    }

    /// Set elevation at grid position
    pub fn set_elevation(&mut self, grid_x: usize, grid_y: usize, elevation: f32) {
        if let Some(idx) = self.grid_index(grid_x, grid_y) {
            self.heightmap[idx] = elevation;
        }
    }

    /// Set terrain type at grid position
    pub fn set_terrain_type(&mut self, grid_x: usize, grid_y: usize, terrain_type: TerrainType) {
        if let Some(idx) = self.grid_index(grid_x, grid_y) {
            self.terrain_types[idx] = terrain_type;
        }
    }

    /// Get elevation at world position (with bilinear interpolation)
    pub fn get_elevation(&self, world_x: f32, world_y: f32) -> f32 {
        let Some((grid_x, grid_y)) = self.world_to_grid(world_x, world_y) else {
            return 0.0;
        };

        // Bilinear interpolation for smooth elevation
        let local_x = (world_x / self.cell_size) - grid_x as f32;
        let local_y = (world_y / self.cell_size) - grid_y as f32;

        let h00 = self.grid_index(grid_x, grid_y)
            .and_then(|idx| self.heightmap.get(idx).copied())
            .unwrap_or(0.0);
        let h10 = self.grid_index(grid_x + 1, grid_y)
            .and_then(|idx| self.heightmap.get(idx).copied())
            .unwrap_or(h00);
        let h01 = self.grid_index(grid_x, grid_y + 1)
            .and_then(|idx| self.heightmap.get(idx).copied())
            .unwrap_or(h00);
        let h11 = self.grid_index(grid_x + 1, grid_y + 1)
            .and_then(|idx| self.heightmap.get(idx).copied())
            .unwrap_or(h00);

        // Bilinear interpolation
        let h0 = h00 * (1.0 - local_x) + h10 * local_x;
        let h1 = h01 * (1.0 - local_x) + h11 * local_x;
        h0 * (1.0 - local_y) + h1 * local_y
    }

    /// Get terrain type at world position
    pub fn get_terrain_type(&self, world_x: f32, world_y: f32) -> TerrainType {
        let Some((grid_x, grid_y)) = self.world_to_grid(world_x, world_y) else {
            return TerrainType::Grass;
        };

        self.grid_index(grid_x, grid_y)
            .and_then(|idx| self.terrain_types.get(idx).copied())
            .unwrap_or(TerrainType::Grass)
    }

    /// Check if line of sight is blocked between two positions
    pub fn has_line_of_sight(&self, from: Vec2, to: Vec2) -> bool {
        let from_elevation = self.get_elevation(from.x, from.y);
        let to_elevation = self.get_elevation(to.x, to.y);

        // Check if terrain blocks LOS
        let steps = 20; // Sample 20 points along the ray
        for i in 1..steps {
            let t = i as f32 / steps as f32;
            let pos = from + (to - from) * t;

            // Check terrain type
            let terrain_type = self.get_terrain_type(pos.x, pos.y);
            if terrain_type.blocks_line_of_sight() {
                return false;
            }

            // Check if elevation blocks view
            let expected_elevation = from_elevation + (to_elevation - from_elevation) * t;
            let actual_elevation = self.get_elevation(pos.x, pos.y);

            // If terrain is 2m higher than expected line, it blocks view
            if actual_elevation > expected_elevation + 2.0 {
                return false;
            }
        }

        true
    }

    /// Get high ground advantage (-1.0 to 1.0, negative = disadvantage)
    pub fn get_high_ground_advantage(&self, shooter_pos: Vec2, target_pos: Vec2) -> f32 {
        let shooter_elevation = self.get_elevation(shooter_pos.x, shooter_pos.y);
        let target_elevation = self.get_elevation(target_pos.x, target_pos.y);

        let elevation_diff = shooter_elevation - target_elevation;

        // Clamp advantage to -1.0 to 1.0
        // Every 10m elevation difference = 0.2 advantage
        (elevation_diff / 10.0 * 0.2).clamp(-1.0, 1.0)
    }

    /// Get movement speed multiplier at position
    pub fn get_movement_speed(&self, pos: Vec2) -> f32 {
        self.get_terrain_type(pos.x, pos.y).movement_speed_modifier()
    }

    /// Get cover effectiveness at position
    pub fn get_cover_effectiveness(&self, pos: Vec2) -> f32 {
        self.get_terrain_type(pos.x, pos.y).cover_effectiveness()
    }

    /// Get raw heightmap data (for rendering)
    pub fn heightmap(&self) -> &[f32] {
        &self.heightmap
    }

    /// Get raw terrain type data (for rendering)
    pub fn terrain_types(&self) -> &[TerrainType] {
        &self.terrain_types
    }

    /// Add a hill at position
    pub fn add_hill(&mut self, center_x: f32, center_y: f32, radius: f32, height: f32) {
        let Some((center_grid_x, center_grid_y)) = self.world_to_grid(center_x, center_y) else {
            return;
        };

        let radius_cells = (radius / self.cell_size).ceil() as i32;

        for dy in -radius_cells..=radius_cells {
            for dx in -radius_cells..=radius_cells {
                let grid_x = (center_grid_x as i32 + dx) as usize;
                let grid_y = (center_grid_y as i32 + dy) as usize;

                if grid_x >= self.grid_width || grid_y >= self.grid_height {
                    continue;
                }

                // Calculate distance from center
                let world_x = grid_x as f32 * self.cell_size;
                let world_y = grid_y as f32 * self.cell_size;
                let dist = ((world_x - center_x).powi(2) + (world_y - center_y).powi(2)).sqrt();

                if dist <= radius {
                    // Smooth falloff using cosine
                    let falloff = (1.0 + (dist / radius * std::f32::consts::PI).cos()) / 2.0;
                    let elevation = height * falloff;

                    if let Some(idx) = self.grid_index(grid_x, grid_y) {
                        self.heightmap[idx] += elevation;

                        // Mark steep areas as hills
                        if elevation > 5.0 {
                            self.terrain_types[idx] = TerrainType::Hill;
                        }
                    }
                }
            }
        }
    }

    /// Add a forest patch
    pub fn add_forest(&mut self, center_x: f32, center_y: f32, radius: f32) {
        let Some((center_grid_x, center_grid_y)) = self.world_to_grid(center_x, center_y) else {
            return;
        };

        let radius_cells = (radius / self.cell_size).ceil() as i32;

        for dy in -radius_cells..=radius_cells {
            for dx in -radius_cells..=radius_cells {
                let grid_x = (center_grid_x as i32 + dx) as usize;
                let grid_y = (center_grid_y as i32 + dy) as usize;

                if grid_x >= self.grid_width || grid_y >= self.grid_height {
                    continue;
                }

                let world_x = grid_x as f32 * self.cell_size;
                let world_y = grid_y as f32 * self.cell_size;
                let dist = ((world_x - center_x).powi(2) + (world_y - center_y).powi(2)).sqrt();

                if dist <= radius {
                    if let Some(idx) = self.grid_index(grid_x, grid_y) {
                        self.terrain_types[idx] = TerrainType::Forest;
                    }
                }
            }
        }
    }

    /// Add a road
    pub fn add_road(&mut self, from_x: f32, from_y: f32, to_x: f32, to_y: f32, width: f32) {
        let steps = ((to_x - from_x).powi(2) + (to_y - from_y).powi(2)).sqrt() / self.cell_size;
        let steps = steps.ceil() as usize;

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = from_x + (to_x - from_x) * t;
            let y = from_y + (to_y - from_y) * t;

            let Some((grid_x, grid_y)) = self.world_to_grid(x, y) else {
                continue;
            };

            let width_cells = (width / self.cell_size).ceil() as i32;

            for dy in -width_cells..=width_cells {
                for dx in -width_cells..=width_cells {
                    let gx = (grid_x as i32 + dx) as usize;
                    let gy = (grid_y as i32 + dy) as usize;

                    if let Some(idx) = self.grid_index(gx, gy) {
                        self.terrain_types[idx] = TerrainType::Road;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_creation() {
        let terrain = Terrain::new(1000.0, 1000.0, 10.0);
        assert_eq!(terrain.grid_dimensions(), (100, 100));
        assert_eq!(terrain.get_elevation(500.0, 500.0), 0.0);
    }

    #[test]
    fn test_hill() {
        let mut terrain = Terrain::new(1000.0, 1000.0, 10.0);
        terrain.add_hill(500.0, 500.0, 100.0, 20.0);

        // Center should be elevated
        assert!(terrain.get_elevation(500.0, 500.0) > 15.0);

        // Edge should be less elevated
        assert!(terrain.get_elevation(600.0, 500.0) < 5.0);
    }

    #[test]
    fn test_line_of_sight() {
        let mut terrain = Terrain::new(1000.0, 1000.0, 10.0);

        // Clear LOS on flat ground
        assert!(terrain.has_line_of_sight(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)));

        // Add forest in the middle
        terrain.add_forest(50.0, 50.0, 20.0);

        // LOS should be blocked
        assert!(!terrain.has_line_of_sight(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0)));
    }

    #[test]
    fn test_high_ground_advantage() {
        let mut terrain = Terrain::new(1000.0, 1000.0, 10.0);
        terrain.add_hill(500.0, 500.0, 100.0, 20.0);

        let high_pos = Vec2::new(500.0, 500.0);
        let low_pos = Vec2::new(700.0, 700.0);

        // Shooting from high ground = advantage
        assert!(terrain.get_high_ground_advantage(high_pos, low_pos) > 0.0);

        // Shooting from low ground = disadvantage
        assert!(terrain.get_high_ground_advantage(low_pos, high_pos) < 0.0);
    }
}

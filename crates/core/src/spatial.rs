//! Spatial partitioning structures for efficient collision and proximity queries

use fxhash::FxHashMap;
use bevy_ecs::entity::Entity;

/// Uniform grid for spatial partitioning
///
/// Provides O(1) insertion/removal and O(k) neighbor queries where k is the
/// number of entities in the queried cells.
pub struct UniformGrid {
    cell_size: f32,
    cells: FxHashMap<(i32, i32), Vec<Entity>>,
}

impl UniformGrid {
    /// Create a new uniform grid with the specified cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: FxHashMap::default(),
        }
    }

    /// Convert world position to cell coordinates
    fn world_to_cell(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x / self.cell_size).floor() as i32,
            (y / self.cell_size).floor() as i32,
        )
    }

    /// Insert an entity at the given position
    pub fn insert(&mut self, entity: Entity, x: f32, y: f32) {
        let cell = self.world_to_cell(x, y);
        self.cells.entry(cell).or_insert_with(Vec::new).push(entity);
    }

    /// Remove an entity from the given position
    pub fn remove(&mut self, entity: Entity, x: f32, y: f32) {
        let cell = self.world_to_cell(x, y);
        if let Some(entities) = self.cells.get_mut(&cell) {
            entities.retain(|&e| e != entity);
            if entities.is_empty() {
                self.cells.remove(&cell);
            }
        }
    }

    /// Move an entity from old position to new position
    pub fn update(&mut self, entity: Entity, old_x: f32, old_y: f32, new_x: f32, new_y: f32) {
        let old_cell = self.world_to_cell(old_x, old_y);
        let new_cell = self.world_to_cell(new_x, new_y);

        if old_cell != new_cell {
            self.remove(entity, old_x, old_y);
            self.insert(entity, new_x, new_y);
        }
    }

    /// Get all entities in the same cell as the given position
    pub fn query_cell(&self, x: f32, y: f32) -> impl Iterator<Item = &Entity> {
        let cell = self.world_to_cell(x, y);
        self.cells
            .get(&cell)
            .map(|v| v.iter())
            .into_iter()
            .flatten()
    }

    /// Get all entities in the cell and its 8 neighbors
    pub fn query_neighbors(&self, x: f32, y: f32) -> Vec<Entity> {
        let (cx, cy) = self.world_to_cell(x, y);
        let mut result = Vec::new();

        for dx in -1..=1 {
            for dy in -1..=1 {
                let cell = (cx + dx, cy + dy);
                if let Some(entities) = self.cells.get(&cell) {
                    result.extend(entities.iter().copied());
                }
            }
        }

        result
    }

    /// Query all entities within a radius
    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        let (cx, cy) = self.world_to_cell(x, y);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        let mut result = Vec::new();

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let cell = (cx + dx, cy + dy);
                if let Some(entities) = self.cells.get(&cell) {
                    // Note: We return all entities in cells that might contain
                    // entities within radius. The caller should do exact distance check.
                    result.extend(entities.iter().copied());
                }
            }
        }

        result
    }

    /// Get all non-empty cells
    pub fn cells(&self) -> impl Iterator<Item = (&(i32, i32), &Vec<Entity>)> {
        self.cells.iter()
    }

    /// Get the number of entities in the grid
    pub fn entity_count(&self) -> usize {
        self.cells.values().map(|v| v.len()).sum()
    }

    /// Get the number of occupied cells
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Clear all entities from the grid
    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

/// Resource for managing the spatial index
#[derive(bevy_ecs::system::Resource)]
pub struct SpatialIndex {
    pub grid: UniformGrid,
}

impl SpatialIndex {
    pub fn new(cell_size: f32) -> Self {
        Self {
            grid: UniformGrid::new(cell_size),
        }
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new(100.0) // Default 100m cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_query() {
        let mut grid = UniformGrid::new(100.0);
        let entity = Entity::from_raw(0);

        grid.insert(entity, 50.0, 50.0);
        let results: Vec<_> = grid.query_cell(50.0, 50.0).copied().collect();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0], entity);
    }

    #[test]
    fn test_remove() {
        let mut grid = UniformGrid::new(100.0);
        let entity = Entity::from_raw(0);

        grid.insert(entity, 50.0, 50.0);
        grid.remove(entity, 50.0, 50.0);
        let results: Vec<_> = grid.query_cell(50.0, 50.0).collect();

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_update_same_cell() {
        let mut grid = UniformGrid::new(100.0);
        let entity = Entity::from_raw(0);

        grid.insert(entity, 50.0, 50.0);
        grid.update(entity, 50.0, 50.0, 60.0, 60.0);

        let results: Vec<_> = grid.query_cell(60.0, 60.0).copied().collect();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_update_different_cell() {
        let mut grid = UniformGrid::new(100.0);
        let entity = Entity::from_raw(0);

        grid.insert(entity, 50.0, 50.0);
        grid.update(entity, 50.0, 50.0, 150.0, 150.0);

        let old_results: Vec<_> = grid.query_cell(50.0, 50.0).collect();
        let new_results: Vec<_> = grid.query_cell(150.0, 150.0).copied().collect();

        assert_eq!(old_results.len(), 0);
        assert_eq!(new_results.len(), 1);
    }

    #[test]
    fn test_query_neighbors() {
        let mut grid = UniformGrid::new(100.0);

        // Insert entities in a 3x3 grid of cells
        for i in 0..9 {
            let entity = Entity::from_raw(i);
            let x = ((i % 3) as f32) * 100.0 + 50.0;
            let y = ((i / 3) as f32) * 100.0 + 50.0;
            grid.insert(entity, x, y);
        }

        // Query center cell should return all 9 neighbors
        let results = grid.query_neighbors(150.0, 150.0);
        assert_eq!(results.len(), 9);
    }
}

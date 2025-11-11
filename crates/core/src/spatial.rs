//! Spatial partitioning structures for efficient collision and proximity queries

use glam::Vec2;
use fxhash::FxHashMap;

/// Uniform grid for spatial partitioning
pub struct UniformGrid {
    cell_size: f32,
    cells: FxHashMap<(i32, i32), Vec<bevy_ecs::entity::Entity>>,
}

impl UniformGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: FxHashMap::default(),
        }
    }
}

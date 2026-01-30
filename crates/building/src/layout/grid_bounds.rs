//! Grid boundary calculations for level-based sizing.
//!
//! The top dungeon level (floor 0) has a 3x3 planning tile grid.
//! Every 2 levels deeper, the grid increases by 2 tiles in each dimension.
//!
//! Level 0: 3x3
//! Level 1: 3x3
//! Level 2: 5x5
//! Level 3: 5x5
//! Level 4: 7x7
//! etc.

use bevy::prelude::*;

/// Base grid size for the top level (floor 0) in planning tiles.
pub const BASE_GRID_SIZE: i32 = 3;

/// How often the grid size increases (every N levels).
pub const SIZE_INCREASE_INTERVAL: usize = 2;

/// How much the grid size increases each interval.
pub const SIZE_INCREASE_AMOUNT: i32 = 2;

/// Calculate the grid size (in planning tiles) for a given floor index.
/// Floor 0 is the top/entrance level.
pub fn grid_size_for_floor(floor_index: usize) -> i32 {
    let increases = (floor_index / SIZE_INCREASE_INTERVAL) as i32;
    BASE_GRID_SIZE + (increases * SIZE_INCREASE_AMOUNT)
}

/// Get the valid grid bounds for a floor in planning tile coordinates.
/// Returns (min, max) where both are inclusive.
/// The grid is centered around (0, 0).
pub fn grid_bounds_for_floor(floor_index: usize) -> (IVec2, IVec2) {
    let size = grid_size_for_floor(floor_index);
    let half = size / 2;
    // For odd sizes (3, 5, 7...), this gives symmetric bounds
    // Size 3: half = 1, bounds = (-1, -1) to (1, 1)
    // Size 5: half = 2, bounds = (-2, -2) to (2, 2)
    let min = IVec2::new(-half, -half);
    let max = IVec2::new(half, half);
    (min, max)
}

/// Check if a position is within the valid grid bounds for a floor.
pub fn is_within_bounds(position: IVec2, floor_index: usize) -> bool {
    let (min, max) = grid_bounds_for_floor(floor_index);
    position.x >= min.x && position.x <= max.x && position.y >= min.y && position.y <= max.y
}

/// Get the world-space bounds for a floor's grid.
/// Returns (min_corner, max_corner) in world coordinates.
pub fn world_bounds_for_floor(floor_index: usize) -> (Vec2, Vec2) {
    use crate::PLANNING_TILE_WORLD_SIZE;

    let (grid_min, grid_max) = grid_bounds_for_floor(floor_index);

    // Convert grid coordinates to world coordinates
    // Grid position (x, y) corresponds to world position at the tile's corner
    let world_min = Vec2::new(
        grid_min.x as f32 * PLANNING_TILE_WORLD_SIZE,
        grid_min.y as f32 * PLANNING_TILE_WORLD_SIZE,
    );
    let world_max = Vec2::new(
        (grid_max.x + 1) as f32 * PLANNING_TILE_WORLD_SIZE,
        (grid_max.y + 1) as f32 * PLANNING_TILE_WORLD_SIZE,
    );

    (world_min, world_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_size_for_floor() {
        assert_eq!(grid_size_for_floor(0), 3);
        assert_eq!(grid_size_for_floor(1), 3);
        assert_eq!(grid_size_for_floor(2), 5);
        assert_eq!(grid_size_for_floor(3), 5);
        assert_eq!(grid_size_for_floor(4), 7);
        assert_eq!(grid_size_for_floor(5), 7);
        assert_eq!(grid_size_for_floor(6), 9);
    }

    #[test]
    fn test_grid_bounds_for_floor() {
        // Floor 0: 3x3 grid, centered
        let (min, max) = grid_bounds_for_floor(0);
        assert_eq!(min, IVec2::new(-1, -1));
        assert_eq!(max, IVec2::new(1, 1));

        // Floor 2: 5x5 grid, centered
        let (min, max) = grid_bounds_for_floor(2);
        assert_eq!(min, IVec2::new(-2, -2));
        assert_eq!(max, IVec2::new(2, 2));

        // Floor 4: 7x7 grid, centered
        let (min, max) = grid_bounds_for_floor(4);
        assert_eq!(min, IVec2::new(-3, -3));
        assert_eq!(max, IVec2::new(3, 3));
    }

    #[test]
    fn test_is_within_bounds() {
        // Floor 0: 3x3 grid, bounds (-1,-1) to (1,1)
        assert!(is_within_bounds(IVec2::ZERO, 0));
        assert!(is_within_bounds(IVec2::new(1, 1), 0));
        assert!(is_within_bounds(IVec2::new(-1, -1), 0));
        assert!(!is_within_bounds(IVec2::new(2, 0), 0));
        assert!(!is_within_bounds(IVec2::new(0, -2), 0));

        // Floor 2: 5x5 grid, bounds (-2,-2) to (2,2)
        assert!(is_within_bounds(IVec2::new(2, 2), 2));
        assert!(!is_within_bounds(IVec2::new(3, 0), 2));
    }
}

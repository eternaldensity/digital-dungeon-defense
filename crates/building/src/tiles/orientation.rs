//! Tile orientation - rotation and flip transforms.

use super::{DoorId, DoorPosition, TileSide, PLANNING_TILE_SIZE};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Orientation of a placed tile (rotation + flip).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct TileOrientation {
    /// Rotation: 0, 1, 2, 3 (90-degree increments clockwise).
    pub rotation: u8,
    /// Horizontal flip (mirror on Y axis, applied before rotation).
    pub flipped: bool,
}

impl TileOrientation {
    pub fn new(rotation: u8, flipped: bool) -> Self {
        Self {
            rotation: rotation % 4,
            flipped,
        }
    }

    /// Rotate clockwise by 90 degrees.
    pub fn rotate_cw(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    /// Rotate counter-clockwise by 90 degrees.
    pub fn rotate_ccw(&mut self) {
        self.rotation = (self.rotation + 3) % 4;
    }

    /// Toggle horizontal flip.
    pub fn flip(&mut self) {
        self.flipped = !self.flipped;
    }

    /// Transform a door ID according to this orientation.
    pub fn transform_door(&self, door: DoorId) -> DoorId {
        let mut side = door.side;
        let mut position = door.position;

        // Apply flip first (mirrors on Y axis, so East/West swap, Left/Right swap)
        if self.flipped {
            side = match side {
                TileSide::North => TileSide::North,
                TileSide::South => TileSide::South,
                TileSide::East => TileSide::West,
                TileSide::West => TileSide::East,
            };
            // For North/South sides, flip also mirrors Left/Right positions
            if matches!(side, TileSide::North | TileSide::South) {
                position = position.mirror();
            }
        }

        // Apply rotation
        for _ in 0..self.rotation {
            let new_side = side.rotate_cw();
            // When rotating, door positions shift based on which wall they're moving to
            position = match (side, new_side) {
                // Moving from horizontal wall to vertical wall
                (TileSide::North, TileSide::East) => flip_position_for_rotation(position),
                (TileSide::South, TileSide::West) => flip_position_for_rotation(position),
                // Moving from vertical wall to horizontal wall
                (TileSide::East, TileSide::South) => flip_position_for_rotation(position),
                (TileSide::West, TileSide::North) => flip_position_for_rotation(position),
                _ => position,
            };
            side = new_side;
        }

        DoorId::new(side, position)
    }

    /// Transform local coordinates (0..SIZE-1) according to this orientation.
    pub fn transform_local_coords(&self, x: usize, y: usize) -> (usize, usize) {
        let size = PLANNING_TILE_SIZE;
        let mut tx = x;
        let mut ty = y;

        // Apply flip first (mirror on Y axis, so X coordinates flip)
        if self.flipped {
            tx = size - 1 - tx;
        }

        // Apply rotation
        for _ in 0..self.rotation {
            let new_x = size - 1 - ty;
            let new_y = tx;
            tx = new_x;
            ty = new_y;
        }

        (tx, ty)
    }

    /// Transform a normalized local position (0.0..1.0) according to this orientation.
    pub fn transform_local_position(&self, pos: Vec2) -> Vec2 {
        let mut p = pos;

        // Apply flip first
        if self.flipped {
            p.x = 1.0 - p.x;
        }

        // Apply rotation
        for _ in 0..self.rotation {
            let new_x = 1.0 - p.y;
            let new_y = p.x;
            p = Vec2::new(new_x, new_y);
        }

        p
    }

    /// Get the number of unique orientations for a tile with given symmetry.
    pub fn unique_count(symmetric_x: bool, symmetric_y: bool) -> usize {
        match (symmetric_x, symmetric_y) {
            (true, true) => 1,   // Fully symmetric (e.g., + shape)
            (true, false) | (false, true) => 2, // One axis of symmetry
            (false, false) => 8, // No symmetry, all 8 orientations unique
        }
    }
}

/// Helper to flip door position during rotation transitions.
fn flip_position_for_rotation(pos: DoorPosition) -> DoorPosition {
    match pos {
        DoorPosition::Left => DoorPosition::Right,
        DoorPosition::Center => DoorPosition::Center,
        DoorPosition::Right => DoorPosition::Left,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_orientation() {
        let o = TileOrientation::default();
        assert_eq!(o.rotation, 0);
        assert!(!o.flipped);
    }

    #[test]
    fn test_rotate_cw() {
        let mut o = TileOrientation::default();
        o.rotate_cw();
        assert_eq!(o.rotation, 1);
        o.rotate_cw();
        assert_eq!(o.rotation, 2);
        o.rotate_cw();
        assert_eq!(o.rotation, 3);
        o.rotate_cw();
        assert_eq!(o.rotation, 0);
    }

    #[test]
    fn test_rotate_ccw() {
        let mut o = TileOrientation::default();
        o.rotate_ccw();
        assert_eq!(o.rotation, 3);
        o.rotate_ccw();
        assert_eq!(o.rotation, 2);
    }

    #[test]
    fn test_transform_door_no_rotation() {
        let o = TileOrientation::default();
        let door = DoorId::new(TileSide::North, DoorPosition::Center);
        let transformed = o.transform_door(door);
        assert_eq!(transformed.side, TileSide::North);
        assert_eq!(transformed.position, DoorPosition::Center);
    }

    #[test]
    fn test_transform_door_90_cw() {
        let o = TileOrientation::new(1, false);
        let door = DoorId::new(TileSide::North, DoorPosition::Center);
        let transformed = o.transform_door(door);
        assert_eq!(transformed.side, TileSide::East);
    }

    #[test]
    fn test_transform_door_180() {
        let o = TileOrientation::new(2, false);
        let door = DoorId::new(TileSide::North, DoorPosition::Left);
        let transformed = o.transform_door(door);
        assert_eq!(transformed.side, TileSide::South);
    }

    #[test]
    fn test_transform_coords_no_rotation() {
        let o = TileOrientation::default();
        assert_eq!(o.transform_local_coords(0, 0), (0, 0));
        assert_eq!(o.transform_local_coords(8, 8), (8, 8));
    }

    #[test]
    fn test_transform_coords_90_cw() {
        let o = TileOrientation::new(1, false);
        // (0,0) -> (8,0) after 90 CW rotation in 9x9 grid
        assert_eq!(o.transform_local_coords(0, 0), (8, 0));
        // (8,0) -> (8,8)
        assert_eq!(o.transform_local_coords(8, 0), (8, 8));
    }

    #[test]
    fn test_transform_coords_flipped() {
        let o = TileOrientation::new(0, true);
        // Flip mirrors X
        assert_eq!(o.transform_local_coords(0, 0), (8, 0));
        assert_eq!(o.transform_local_coords(8, 0), (0, 0));
    }
}

//! Tile template definitions - the "blueprint" for a tile type.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Size of planning tiles in small tiles (9x9).
pub const PLANNING_TILE_SIZE: usize = 9;

/// Door position indices within a tile side (0-indexed into 9 positions).
pub const DOOR_LEFT_INDEX: usize = 1;
pub const DOOR_CENTER_INDEX: usize = 4;
pub const DOOR_RIGHT_INDEX: usize = 7;

/// Unique identifier for a tile template.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct TileTemplateId(pub u32);

/// Position of a door along a tile edge.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum DoorPosition {
    /// Index 1 (second position)
    Left,
    /// Index 4 (center position)
    Center,
    /// Index 7 (eighth position)
    Right,
}

impl DoorPosition {
    /// Get the small-tile index for this door position.
    pub fn index(self) -> usize {
        match self {
            DoorPosition::Left => DOOR_LEFT_INDEX,
            DoorPosition::Center => DOOR_CENTER_INDEX,
            DoorPosition::Right => DOOR_RIGHT_INDEX,
        }
    }

    /// Mirror the position (for matching doors on opposite sides).
    pub fn mirror(self) -> Self {
        match self {
            DoorPosition::Left => DoorPosition::Right,
            DoorPosition::Center => DoorPosition::Center,
            DoorPosition::Right => DoorPosition::Left,
        }
    }

    /// All door positions.
    pub fn all() -> [DoorPosition; 3] {
        [DoorPosition::Left, DoorPosition::Center, DoorPosition::Right]
    }
}

/// Which side of the tile a door is on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum TileSide {
    North,
    East,
    South,
    West,
}

impl TileSide {
    /// Get the opposite side.
    pub fn opposite(self) -> Self {
        match self {
            TileSide::North => TileSide::South,
            TileSide::East => TileSide::West,
            TileSide::South => TileSide::North,
            TileSide::West => TileSide::East,
        }
    }

    /// Rotate clockwise by 90 degrees.
    pub fn rotate_cw(self) -> Self {
        match self {
            TileSide::North => TileSide::East,
            TileSide::East => TileSide::South,
            TileSide::South => TileSide::West,
            TileSide::West => TileSide::North,
        }
    }

    /// Rotate counter-clockwise by 90 degrees.
    pub fn rotate_ccw(self) -> Self {
        match self {
            TileSide::North => TileSide::West,
            TileSide::East => TileSide::North,
            TileSide::South => TileSide::East,
            TileSide::West => TileSide::South,
        }
    }

    /// All sides in order.
    pub fn all() -> [TileSide; 4] {
        [TileSide::North, TileSide::East, TileSide::South, TileSide::West]
    }

    /// Get the direction vector for this side (pointing outward).
    pub fn direction(self) -> IVec2 {
        match self {
            TileSide::North => IVec2::Y,
            TileSide::East => IVec2::X,
            TileSide::South => IVec2::NEG_Y,
            TileSide::West => IVec2::NEG_X,
        }
    }
}

/// A specific door location on a tile.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct DoorId {
    pub side: TileSide,
    pub position: DoorPosition,
}

impl DoorId {
    pub fn new(side: TileSide, position: DoorPosition) -> Self {
        Self { side, position }
    }

    /// Get the door that would connect to this one from an adjacent tile.
    pub fn matching_door(self) -> Self {
        Self {
            side: self.side.opposite(),
            position: self.position.mirror(),
        }
    }
}

/// Types of construction slots within a tile.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SlotType {
    /// Can place traps
    Trap,
    /// Can place creature spawners
    Spawner,
    /// Dungeon core placement (only one in entire dungeon)
    Core,
    /// Connection to the floor above
    UpConnector,
    /// Connection to the floor below
    DownConnector,
    /// Cosmetic only
    Decoration,
}

/// A construction slot within a tile template.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstructionSlot {
    /// Local position within the tile (0.0 - 1.0 normalized coordinates).
    pub local_position: Vec2,
    /// What can be placed here.
    pub slot_type: SlotType,
    /// Unique ID within this template.
    pub slot_id: u8,
}

impl ConstructionSlot {
    pub fn new(x: f32, y: f32, slot_type: SlotType, slot_id: u8) -> Self {
        Self {
            local_position: Vec2::new(x, y),
            slot_type,
            slot_id,
        }
    }
}

/// Node in the internal connectivity graph.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ConnectivityNode {
    /// A door on the tile edge.
    Door(DoorId),
    /// A construction slot (by slot_id).
    Slot(u8),
}

/// Defines a tile's shape, doors, and internal structure.
#[derive(Clone, Debug)]
pub struct TileTemplate {
    pub id: TileTemplateId,
    pub name: String,

    /// Which doors exist on this template (before orientation transform).
    pub doors: HashSet<DoorId>,

    /// Construction slots available.
    pub slots: Vec<ConstructionSlot>,

    /// Sets of nodes that are internally connected.
    /// Each HashSet represents a connected component within the tile.
    /// Nodes in the same set can reach each other; nodes in different sets cannot.
    pub connectivity: Vec<HashSet<ConnectivityNode>>,

    /// Small-tile walkability pattern (9x9 grid).
    /// `true` = walkable, `false` = wall.
    /// Indexed as `walkable_pattern[y][x]`.
    pub walkable_pattern: [[bool; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE],

    /// Visual identifier for rendering.
    pub visual_id: String,
}

impl TileTemplate {
    /// Check if two nodes are connected within this tile.
    pub fn are_connected(&self, a: ConnectivityNode, b: ConnectivityNode) -> bool {
        self.connectivity
            .iter()
            .any(|set| set.contains(&a) && set.contains(&b))
    }

    /// Get all nodes connected to a given node.
    pub fn connected_to(&self, node: ConnectivityNode) -> HashSet<ConnectivityNode> {
        for set in &self.connectivity {
            if set.contains(&node) {
                return set.clone();
            }
        }
        HashSet::new()
    }

    /// Check if this tile has a door at the given location.
    pub fn has_door(&self, door: DoorId) -> bool {
        self.doors.contains(&door)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_door_position_mirror() {
        assert_eq!(DoorPosition::Left.mirror(), DoorPosition::Right);
        assert_eq!(DoorPosition::Center.mirror(), DoorPosition::Center);
        assert_eq!(DoorPosition::Right.mirror(), DoorPosition::Left);
    }

    #[test]
    fn test_tile_side_opposite() {
        assert_eq!(TileSide::North.opposite(), TileSide::South);
        assert_eq!(TileSide::East.opposite(), TileSide::West);
        assert_eq!(TileSide::South.opposite(), TileSide::North);
        assert_eq!(TileSide::West.opposite(), TileSide::East);
    }

    #[test]
    fn test_tile_side_rotate() {
        assert_eq!(TileSide::North.rotate_cw(), TileSide::East);
        assert_eq!(TileSide::East.rotate_cw(), TileSide::South);
        assert_eq!(TileSide::South.rotate_cw(), TileSide::West);
        assert_eq!(TileSide::West.rotate_cw(), TileSide::North);
    }

    #[test]
    fn test_matching_door() {
        let door = DoorId::new(TileSide::North, DoorPosition::Left);
        let matching = door.matching_door();
        assert_eq!(matching.side, TileSide::South);
        assert_eq!(matching.position, DoorPosition::Right);
    }
}

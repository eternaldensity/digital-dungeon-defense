//! Placed tiles and features - instances in the dungeon layout.

use super::{TileOrientation, TileTemplateId};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// What's been placed in a construction slot.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum PlacedFeature {
    #[default]
    Empty,
    Trap {
        trap_type: TrapType,
    },
    Spawner {
        spawner_type: SpawnerType,
    },
    Core,
    UpConnector,
    DownConnector,
}

impl PlacedFeature {
    pub fn is_empty(&self) -> bool {
        matches!(self, PlacedFeature::Empty)
    }

    pub fn is_core(&self) -> bool {
        matches!(self, PlacedFeature::Core)
    }

    pub fn is_up_connector(&self) -> bool {
        matches!(self, PlacedFeature::UpConnector)
    }

    pub fn is_down_connector(&self) -> bool {
        matches!(self, PlacedFeature::DownConnector)
    }

    pub fn is_connector(&self) -> bool {
        self.is_up_connector() || self.is_down_connector()
    }
}

/// Types of traps that can be placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrapType {
    Firewall,
    DataSpike,
}

/// Types of spawners that can be placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnerType {
    FloofByte,
}

/// An instance of a tile placed in the floor layout.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlacedTile {
    /// Which template this tile is based on.
    pub template_id: TileTemplateId,
    /// Rotation and flip state.
    pub orientation: TileOrientation,
    /// Position in the planning grid (not small-tile grid).
    pub position: IVec2,
    /// What's placed in each slot (indexed by slot_id).
    pub placed_features: HashMap<u8, PlacedFeature>,
}

impl PlacedTile {
    pub fn new(template_id: TileTemplateId, position: IVec2) -> Self {
        Self {
            template_id,
            orientation: TileOrientation::default(),
            position,
            placed_features: HashMap::new(),
        }
    }

    pub fn with_orientation(mut self, orientation: TileOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Place a feature in a slot.
    pub fn place_feature(&mut self, slot_id: u8, feature: PlacedFeature) {
        self.placed_features.insert(slot_id, feature);
    }

    /// Remove a feature from a slot.
    pub fn remove_feature(&mut self, slot_id: u8) {
        self.placed_features.remove(&slot_id);
    }

    /// Get the feature at a slot, or Empty if none.
    pub fn get_feature(&self, slot_id: u8) -> &PlacedFeature {
        self.placed_features
            .get(&slot_id)
            .unwrap_or(&PlacedFeature::Empty)
    }

    /// Check if this tile contains the dungeon core.
    pub fn has_core(&self) -> bool {
        self.placed_features.values().any(|f| f.is_core())
    }

    /// Check if this tile has an up connector.
    pub fn has_up_connector(&self) -> bool {
        self.placed_features.values().any(|f| f.is_up_connector())
    }

    /// Check if this tile has a down connector.
    pub fn has_down_connector(&self) -> bool {
        self.placed_features.values().any(|f| f.is_down_connector())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placed_tile_new() {
        let tile = PlacedTile::new(TileTemplateId(1), IVec2::new(0, 0));
        assert_eq!(tile.template_id, TileTemplateId(1));
        assert_eq!(tile.position, IVec2::ZERO);
        assert!(tile.placed_features.is_empty());
    }

    #[test]
    fn test_place_and_get_feature() {
        let mut tile = PlacedTile::new(TileTemplateId(1), IVec2::ZERO);
        tile.place_feature(0, PlacedFeature::Core);

        assert!(tile.get_feature(0).is_core());
        assert!(tile.get_feature(1).is_empty());
        assert!(tile.has_core());
    }

    #[test]
    fn test_remove_feature() {
        let mut tile = PlacedTile::new(TileTemplateId(1), IVec2::ZERO);
        tile.place_feature(0, PlacedFeature::Core);
        tile.remove_feature(0);

        assert!(tile.get_feature(0).is_empty());
        assert!(!tile.has_core());
    }
}

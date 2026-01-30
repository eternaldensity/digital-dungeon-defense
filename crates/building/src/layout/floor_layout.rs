//! Floor layout - a single dungeon floor's tile arrangement.

use crate::tiles::{PlacedTile, TileTemplateId};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation errors for a floor layout.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ValidationError {
    /// Two adjacent tiles have mismatched doors.
    MismatchedDoors {
        tile1: IVec2,
        tile2: IVec2,
        message: String,
    },
    /// A door leads to empty space (no adjacent tile).
    DoorToVoid { tile: IVec2, message: String },
    /// No valid path between connectors.
    NoPath { from: IVec2, to: IVec2 },
    /// Missing required feature.
    MissingCore,
    /// Floor 0 needs a down connector or core.
    MissingDownConnectorOrCore,
    /// Non-bottom floor needs a down connector.
    MissingDownConnector,
    /// Non-top floor needs an up connector.
    MissingUpConnector,
    /// Tile overlap at position.
    TileOverlap { position: IVec2 },
}

/// A floor's tile-based layout.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FloorLayout {
    /// Which floor this is (0 = bottom/core floor).
    pub floor_index: usize,
    /// Tiles placed on this floor, keyed by planning grid position.
    pub tiles: HashMap<IVec2, PlacedTile>,
    /// Whether the current layout is valid.
    pub is_valid: bool,
    /// Current validation errors.
    pub validation_errors: Vec<ValidationError>,
}

impl FloorLayout {
    pub fn new(floor_index: usize) -> Self {
        Self {
            floor_index,
            tiles: HashMap::new(),
            is_valid: false,
            validation_errors: Vec::new(),
        }
    }

    /// Place a tile at a position. Returns the previous tile if any.
    pub fn place_tile(&mut self, position: IVec2, tile: PlacedTile) -> Option<PlacedTile> {
        self.is_valid = false; // Needs re-validation
        self.tiles.insert(position, tile)
    }

    /// Remove a tile at a position.
    pub fn remove_tile(&mut self, position: IVec2) -> Option<PlacedTile> {
        self.is_valid = false;
        self.tiles.remove(&position)
    }

    /// Get a tile at a position.
    pub fn get_tile(&self, position: IVec2) -> Option<&PlacedTile> {
        self.tiles.get(&position)
    }

    /// Get a mutable reference to a tile at a position.
    pub fn get_tile_mut(&mut self, position: IVec2) -> Option<&mut PlacedTile> {
        self.is_valid = false; // Assume mutation invalidates
        self.tiles.get_mut(&position)
    }

    /// Check if a position has a tile.
    pub fn has_tile(&self, position: IVec2) -> bool {
        self.tiles.contains_key(&position)
    }

    /// Get all tile positions.
    pub fn tile_positions(&self) -> impl Iterator<Item = &IVec2> {
        self.tiles.keys()
    }

    /// Check if this floor has a core placed.
    pub fn has_core(&self) -> bool {
        self.tiles.values().any(|t| t.has_core())
    }

    /// Check if this floor has an up connector.
    pub fn has_up_connector(&self) -> bool {
        self.tiles.values().any(|t| t.has_up_connector())
    }

    /// Check if this floor has a down connector.
    pub fn has_down_connector(&self) -> bool {
        self.tiles.values().any(|t| t.has_down_connector())
    }

    /// Find tile positions with a specific template.
    pub fn find_tiles_with_template(&self, template_id: TileTemplateId) -> Vec<IVec2> {
        self.tiles
            .iter()
            .filter(|(_, t)| t.template_id == template_id)
            .map(|(pos, _)| *pos)
            .collect()
    }

    /// Get the bounding box of all placed tiles.
    pub fn bounds(&self) -> Option<(IVec2, IVec2)> {
        if self.tiles.is_empty() {
            return None;
        }

        let mut min = IVec2::new(i32::MAX, i32::MAX);
        let mut max = IVec2::new(i32::MIN, i32::MIN);

        for pos in self.tiles.keys() {
            min.x = min.x.min(pos.x);
            min.y = min.y.min(pos.y);
            max.x = max.x.max(pos.x);
            max.y = max.y.max(pos.y);
        }

        Some((min, max))
    }

    /// Mark validation results.
    pub fn set_validation_result(&mut self, errors: Vec<ValidationError>) {
        self.validation_errors = errors;
        self.is_valid = self.validation_errors.is_empty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_floor() {
        let floor = FloorLayout::new(0);
        assert_eq!(floor.floor_index, 0);
        assert!(floor.tiles.is_empty());
        assert!(!floor.is_valid);
    }

    #[test]
    fn test_place_and_get_tile() {
        let mut floor = FloorLayout::new(0);
        let tile = PlacedTile::new(TileTemplateId(1), IVec2::ZERO);

        floor.place_tile(IVec2::ZERO, tile);

        assert!(floor.has_tile(IVec2::ZERO));
        assert!(!floor.has_tile(IVec2::ONE));

        let retrieved = floor.get_tile(IVec2::ZERO);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().template_id, TileTemplateId(1));
    }

    #[test]
    fn test_remove_tile() {
        let mut floor = FloorLayout::new(0);
        floor.place_tile(IVec2::ZERO, PlacedTile::new(TileTemplateId(1), IVec2::ZERO));

        let removed = floor.remove_tile(IVec2::ZERO);
        assert!(removed.is_some());
        assert!(!floor.has_tile(IVec2::ZERO));
    }

    #[test]
    fn test_bounds() {
        let mut floor = FloorLayout::new(0);
        assert!(floor.bounds().is_none());

        floor.place_tile(IVec2::new(-1, -1), PlacedTile::new(TileTemplateId(1), IVec2::new(-1, -1)));
        floor.place_tile(IVec2::new(2, 3), PlacedTile::new(TileTemplateId(1), IVec2::new(2, 3)));

        let (min, max) = floor.bounds().unwrap();
        assert_eq!(min, IVec2::new(-1, -1));
        assert_eq!(max, IVec2::new(2, 3));
    }
}

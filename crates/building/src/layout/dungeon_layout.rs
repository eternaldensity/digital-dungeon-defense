//! Dungeon layout - the complete dungeon structure using planning tiles.

use super::FloorLayout;
use crate::tiles::{create_default_templates, TileTemplate, TileTemplateId};
use bevy::prelude::*;
use std::collections::HashMap;

/// The complete dungeon layout using the planning tile system.
#[derive(Resource, Clone, Debug)]
pub struct DungeonLayout {
    /// All floor layouts.
    pub floors: Vec<FloorLayout>,
    /// Template registry.
    pub templates: HashMap<TileTemplateId, TileTemplate>,
}

impl Default for DungeonLayout {
    fn default() -> Self {
        Self {
            floors: vec![FloorLayout::new(0)],
            templates: create_default_templates(),
        }
    }
}

impl DungeonLayout {
    /// Create a new dungeon layout with default templates.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a floor by index.
    pub fn get_floor(&self, index: usize) -> Option<&FloorLayout> {
        self.floors.get(index)
    }

    /// Get a mutable floor by index.
    pub fn get_floor_mut(&mut self, index: usize) -> Option<&mut FloorLayout> {
        self.floors.get_mut(index)
    }

    /// Add a new floor. Returns the new floor's index.
    pub fn add_floor(&mut self) -> usize {
        let index = self.floors.len();
        self.floors.push(FloorLayout::new(index));
        index
    }

    /// Get the number of floors.
    pub fn floor_count(&self) -> usize {
        self.floors.len()
    }

    /// Get a template by ID.
    pub fn get_template(&self, id: TileTemplateId) -> Option<&TileTemplate> {
        self.templates.get(&id)
    }

    /// Check if the entire dungeon is valid.
    pub fn is_valid(&self) -> bool {
        self.floors.iter().all(|f| f.is_valid)
    }

    /// Check if the dungeon has a core placed.
    pub fn has_core(&self) -> bool {
        self.floors.iter().any(|f| f.has_core())
    }

    /// Find which floor has the core.
    pub fn core_floor(&self) -> Option<usize> {
        self.floors.iter().position(|f| f.has_core())
    }

    /// Get the bounding box across all floors in planning grid coordinates.
    pub fn bounds(&self) -> Option<(IVec2, IVec2)> {
        let mut min = IVec2::new(i32::MAX, i32::MAX);
        let mut max = IVec2::new(i32::MIN, i32::MIN);
        let mut found_any = false;

        for floor in &self.floors {
            if let Some((floor_min, floor_max)) = floor.bounds() {
                found_any = true;
                min.x = min.x.min(floor_min.x);
                min.y = min.y.min(floor_min.y);
                max.x = max.x.max(floor_max.x);
                max.y = max.y.max(floor_max.y);
            }
        }

        if found_any {
            Some((min, max))
        } else {
            None
        }
    }
}

/// Current building mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, States, Hash)]
pub enum BuildMode {
    /// Not in build mode (combat/view).
    #[default]
    Disabled,
    /// Viewing the dungeon layout.
    View,
    /// Selecting a tile from the palette.
    TileSelect,
    /// Placing the selected tile.
    TilePlace,
    /// Placing features in slots.
    FeaturePlace,
    /// Removing tiles.
    Demolish,
}

/// Build mode state resource.
#[derive(Resource, Default)]
pub struct BuildModeState {
    /// Currently selected tile template.
    pub selected_template: Option<TileTemplateId>,
    /// Current orientation for placement.
    pub current_orientation: crate::tiles::TileOrientation,
    /// Position being hovered for preview.
    pub preview_position: Option<IVec2>,
    /// Currently selected placed tile (for feature placement).
    pub selected_tile: Option<IVec2>,
    /// Active floor being edited.
    pub active_floor: usize,
    /// Selected slot for feature placement.
    pub selected_slot: Option<u8>,
}

impl BuildModeState {
    /// Reset selection state.
    pub fn clear_selection(&mut self) {
        self.selected_template = None;
        self.preview_position = None;
        self.selected_tile = None;
        self.selected_slot = None;
    }

    /// Switch to a different floor.
    pub fn set_active_floor(&mut self, floor: usize) {
        self.active_floor = floor;
        self.selected_tile = None;
        self.selected_slot = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_layout() {
        let layout = DungeonLayout::default();
        assert_eq!(layout.floor_count(), 1);
        assert!(!layout.templates.is_empty());
    }

    #[test]
    fn test_add_floor() {
        let mut layout = DungeonLayout::new();
        assert_eq!(layout.floor_count(), 1);

        let new_index = layout.add_floor();
        assert_eq!(new_index, 1);
        assert_eq!(layout.floor_count(), 2);
    }

    #[test]
    fn test_get_template() {
        let layout = DungeonLayout::new();
        let template = layout.get_template(TileTemplateId(1));
        assert!(template.is_some());
        assert_eq!(template.unwrap().name, "Basic Room");
    }

    #[test]
    fn test_build_mode_state() {
        let mut state = BuildModeState::default();
        state.selected_template = Some(TileTemplateId(1));
        state.selected_tile = Some(IVec2::ZERO);

        state.clear_selection();
        assert!(state.selected_template.is_none());
        assert!(state.selected_tile.is_none());
    }
}

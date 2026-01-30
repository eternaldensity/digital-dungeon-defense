//! Building system for Digital Dungeon Defense.
//!
//! This crate provides a tile-based dungeon building system where floors are
//! composed of large planning tiles (9x9 small tiles) that compile down to
//! the gameplay grid for pathfinding and combat.
//!
//! # Core Concepts
//!
//! - **Planning Tiles**: Large rooms/hallways (9x9 small tiles each, 288x288 world units)
//! - **Doors**: Each tile side has 3 potential door positions (Left, Center, Right)
//! - **Orientation**: Tiles can be rotated (4 states) and flipped (2 states) = 8 orientations
//! - **Construction Slots**: Locations within tiles for placing features (traps, spawners, etc.)
//! - **Connectivity**: Internal graph defining which doors/slots can reach each other
//!
//! # Validation
//!
//! The system validates:
//! - Door matching between adjacent tiles
//! - Path connectivity between floor connectors
//! - Required features (core, up/down connectors)

pub mod input;
pub mod layout;
pub mod tiles;
pub mod validation;

pub use input::{
    planning_grid_to_world, world_to_planning_grid, BuildInputPlugin, BuildModeChangedEvent,
    TilePlacedEvent, TileRemovedEvent, PLANNING_TILE_WORLD_SIZE,
};
pub use layout::{BuildMode, BuildModeState, DungeonLayout, FloorLayout, ValidationError};
pub use tiles::{
    ConnectivityNode, ConstructionSlot, DoorId, DoorPosition, PlacedFeature, PlacedTile,
    SlotType, TileOrientation, TileSide, TileTemplate, TileTemplateId, PLANNING_TILE_SIZE,
};
pub use validation::{
    can_place_tile, validate_door_matching, validate_dungeon, validate_floor_connectivity,
};

use bevy::prelude::*;

/// Plugin for the building system.
pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<BuildMode>()
            .init_resource::<DungeonLayout>()
            .init_resource::<BuildModeState>()
            .add_plugins(BuildInputPlugin);
    }
}

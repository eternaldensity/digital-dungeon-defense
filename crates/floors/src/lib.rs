//! Floors crate - Dungeon floor types with modifiers
//!
//! Each floor type is a self-contained plugin. Adding a new floor type:
//! 1. Create a new module (e.g., `ice_floor.rs`)
//! 2. Define the floor's components, modifiers, and systems
//! 3. Create a plugin struct that registers everything
//! 4. Add the plugin to FloorsPlugin
//!
//! Floor types apply modifiers to entities on that floor and can
//! also affect adjacent floors.

mod modifiers;
mod fire_floor;
mod ice_floor;

pub use modifiers::*;
pub use fire_floor::*;
pub use ice_floor::*;

use bevy::prelude::*;

/// Main plugin that aggregates all floor type plugins
pub struct FloorsPlugin;

impl Plugin for FloorsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Shared modifier systems
            .add_plugins(ModifiersPlugin)

            // Floor types
            .add_plugins((
                FireFloorPlugin,
                IceFloorPlugin,
                // Add new floor types here
            ));
    }
}

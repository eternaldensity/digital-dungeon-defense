//! Creatures crate - FloofBytes (player creatures) and Enemies
//!
//! Each creature type is a self-contained plugin. Adding a new creature:
//! 1. Create a new module (e.g., `floofbytes/spark_floof.rs`)
//! 2. Define the creature's components and systems
//! 3. Create a plugin struct that registers everything
//! 4. Add the plugin to CreaturesPlugin

mod spawner;
mod floofbytes;
mod enemies;

pub use spawner::*;
pub use floofbytes::*;
pub use enemies::*;

use bevy::prelude::*;

/// Main plugin that aggregates all creature plugins
pub struct CreaturesPlugin;

impl Plugin for CreaturesPlugin {
    fn build(&self, app: &mut App) {
        app
            // Core spawner system
            .add_plugins(SpawnerPlugin)

            // FloofBytes (player creatures)
            .add_plugins((
                BasicFloofPlugin,
                // Add new FloofBytes here
            ))

            // Enemies
            .add_plugins((
                VirusPlugin,
                // Add new enemies here
            ));
    }
}

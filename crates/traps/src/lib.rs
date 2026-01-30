//! Traps crate - Dungeon traps and defenses
//!
//! Each trap type is a self-contained plugin. Adding a new trap:
//! 1. Create a new module (e.g., `data_spike.rs`)
//! 2. Define the trap's components, triggers, and systems
//! 3. Create a plugin struct that registers everything
//! 4. Add the plugin to TrapsPlugin
//!
//! Traps have a reset cost in bytes and can be damaged by enemies.

mod trap_base;
mod firewall;
mod data_spike;

pub use trap_base::*;
pub use firewall::*;
pub use data_spike::*;

use bevy::prelude::*;

/// Main plugin that aggregates all trap plugins
pub struct TrapsPlugin;

impl Plugin for TrapsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Base trap systems
            .add_plugins(TrapBasePlugin)

            // Trap types
            .add_plugins((
                FirewallPlugin,
                DataSpikePlugin,
                // Add new traps here
            ));
    }
}

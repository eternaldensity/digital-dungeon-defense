//! Core systems and components for Digital Dungeon Defense
//!
//! This crate contains shared functionality used across all game systems:
//! - Resource types (Bytes, Floof, Chips)
//! - Health and damage systems
//! - Combat tags for research unlocks
//! - Pathfinding
//! - Dungeon grid management

mod resources;
mod health;
mod combat;
mod dungeon;
mod pathfinding;

pub use resources::*;
pub use health::*;
pub use combat::*;
pub use dungeon::*;
pub use pathfinding::*;

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<PlayerResources>()
            .init_resource::<DungeonGrid>()

            // Register types for reflection/serialization
            .register_type::<Health>()
            .register_type::<MovementSpeed>()
            .register_type::<CombatStats>()
            .register_type::<Lifetime>()
            .register_type::<Team>()

            // Events
            .add_event::<DamageEvent>()
            .add_event::<DeathEvent>()
            .add_event::<ResourceGainEvent>()

            // Core systems
            .add_systems(Update, (
                apply_damage_system,
                check_death_system,
                lifetime_system,
                collect_floof_system,
                passive_byte_generation_system,
            ).chain())

            // Startup
            .add_systems(Startup, setup_dungeon);
    }
}

//! Core systems and components for Digital Dungeon Defense
//!
//! This crate contains shared functionality used across all game systems:
//! - Resource types (Bytes, Floof, Chips)
//! - Health and damage systems
//! - Combat tags for research unlocks
//! - Pathfinding
//! - Dungeon grid management
//! - Game state management

mod combat;
mod dungeon;
mod game_state;
mod health;
mod pathfinding;
mod resources;
mod waves;

pub use combat::*;
pub use dungeon::*;
pub use game_state::*;
pub use health::*;
pub use pathfinding::*;
pub use resources::*;
pub use waves::*;

use bevy::prelude::*;

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            // Game state
            .init_state::<GameState>()
            .init_resource::<GameOverState>()

            // Resources
            .init_resource::<PlayerResources>()
            .init_resource::<DungeonGrid>()

            // Register types for reflection/serialization
            .register_type::<Health>()
            .register_type::<MovementSpeed>()
            .register_type::<CombatStats>()
            .register_type::<Lifetime>()
            .register_type::<Team>()

            // Messages (formerly events)
            .add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_message::<ResourceGainEvent>()
            .add_message::<WaveStartEvent>()
            .add_message::<WaveCompleteEvent>()
            .add_message::<SpawnWaveEnemyEvent>()
            .add_message::<GameOverEvent>()

            // Wave state resource
            .init_resource::<WaveState>()

            // Register combat types
            .register_type::<AttackCooldown>()

            // Title screen input
            .add_systems(Update, title_input_system.run_if(in_state(GameState::Title)))

            // Game over input
            .add_systems(Update, game_over_input_system.run_if(in_state(GameState::GameOver)))

            // Pause toggle (Active or Paused)
            .add_systems(Update, pause_toggle_system.run_if(
                in_state(GameState::Active).or(in_state(GameState::Paused))
            ))

            // Active gameplay systems - only run during Active state
            .add_systems(Update, (
                apply_damage_system,
                check_death_system,
                lifetime_system,
                collect_floof_system,
                passive_byte_generation_system,
                auto_attack_system,
                wave_state_system,
                wave_spawn_system,
                core_death_system,
            ).chain().run_if(in_state(GameState::Active)));
    }
}

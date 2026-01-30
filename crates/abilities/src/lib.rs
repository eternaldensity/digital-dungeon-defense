//! Abilities crate - Self-contained ability plugins
//!
//! Each ability is a self-contained plugin. Adding a new ability:
//! 1. Create a new module (e.g., `damage/shock.rs`)
//! 2. Define the ability component, events, and systems
//! 3. Create a plugin struct that registers everything
//! 4. Add the plugin to AbilitiesPlugin
//!
//! Abilities are attached to entities as components. An entity with
//! both Fireball and Heal components has access to both abilities.

mod damage;
mod support;

pub use damage::*;
pub use support::*;

use bevy::prelude::*;
use ddd_core::DamageType;

/// Main plugin that aggregates all ability plugins
pub struct AbilitiesPlugin;

impl Plugin for AbilitiesPlugin {
    fn build(&self, app: &mut App) {
        app
            // Damage abilities
            .add_plugins((
                FireballPlugin,
                // Add new damage abilities here
            ))
            // Support abilities
            .add_plugins((
                HealPlugin,
                // Add new support abilities here
            ));
    }
}

/// Trait for ability data - allows data-driven configuration
pub trait AbilityData: Send + Sync + 'static {
    fn cooldown(&self) -> f32;
    fn resource_cost(&self) -> ResourceCost;
}

/// Resource cost for using an ability
#[derive(Clone, Default)]
pub struct ResourceCost {
    pub bytes: f32,
    pub floof: f32,
}

/// Marker for abilities that deal damage
#[derive(Component)]
pub struct DealsDamage {
    pub base_damage: f32,
    pub damage_type: DamageType,
}

/// Marker for abilities that can target enemies
#[derive(Component)]
pub struct TargetsEnemies;

/// Marker for abilities that can target allies
#[derive(Component)]
pub struct TargetsAllies;

/// Marker for abilities that are area-of-effect
#[derive(Component)]
pub struct AreaOfEffect {
    pub radius: f32,
}

/// Cooldown component shared by all abilities
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AbilityCooldown {
    pub timer: Timer,
    pub ready: bool,
}

impl AbilityCooldown {
    pub fn new(seconds: f32) -> Self {
        let mut timer = Timer::from_seconds(seconds, TimerMode::Once);
        timer.set_elapsed(timer.duration()); // Start ready
        Self { timer, ready: true }
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        self.timer.tick(delta);
        if self.timer.is_finished() {
            self.ready = true;
        }
    }

    pub fn trigger(&mut self) {
        self.timer.reset();
        self.ready = false;
    }
}

//! Generic spawner system for creatures
//!
//! Spawners create a limited pool of creatures that respawn after death/timeout.
//! Each individual creature maintains its identity and experience across respawns.

use bevy::prelude::*;
use bevy::ecs::message::{Message, MessageWriter};
use ddd_core::{CreatureExperience, Team};

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Spawner>()
            .add_message::<SpawnCreatureEvent>()
            .add_systems(Update, (
                spawner_tick_system,
                respawn_tracking_system,
            ));
    }
}

/// A spawner that creates creatures
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Spawner {
    /// Maximum number of creatures this spawner can have alive at once
    pub max_creatures: usize,
    /// Time between spawns
    pub spawn_interval: Timer,
    /// Cost in bytes to activate/reset
    pub byte_cost: f32,
    /// Whether the spawner is currently active
    pub active: bool,
    /// Currently alive creatures spawned by this spawner
    pub spawned_creatures: Vec<Entity>,
}

impl Default for Spawner {
    fn default() -> Self {
        Self {
            max_creatures: 3,
            spawn_interval: Timer::from_seconds(5.0, TimerMode::Repeating),
            byte_cost: 10.0,
            active: true,
            spawned_creatures: Vec::new(),
        }
    }
}

impl Spawner {
    pub fn new(max_creatures: usize, spawn_interval: f32, byte_cost: f32) -> Self {
        Self {
            max_creatures,
            spawn_interval: Timer::from_seconds(spawn_interval, TimerMode::Repeating),
            byte_cost,
            active: true,
            spawned_creatures: Vec::new(),
        }
    }

    pub fn can_spawn(&self) -> bool {
        self.active && self.spawned_creatures.len() < self.max_creatures
    }
}

/// Marker trait for creature types that can be spawned
/// Implement this on your creature marker component
pub trait SpawnableCreature: Component + Default {
    /// Spawn a new instance of this creature
    fn spawn(commands: &mut Commands, position: Vec3, team: Team) -> Entity;
}

/// Message to request spawning a specific creature type
#[derive(Message)]
pub struct SpawnCreatureEvent {
    pub spawner: Entity,
    pub position: Vec3,
    pub team: Team,
}

/// Links a creature back to its spawner for respawn tracking
#[derive(Component)]
pub struct SpawnedBy(pub Entity);

/// Stored experience for respawning creatures
#[derive(Component, Default)]
pub struct StoredCreatureData {
    pub experience: CreatureExperience,
    pub respawn_timer: Timer,
    pub slot_index: usize,
}

/// System that ticks spawners and emits spawn events
pub fn spawner_tick_system(
    mut spawners: Query<(Entity, &mut Spawner, &Transform)>,
    time: Res<Time>,
    mut spawn_events: MessageWriter<SpawnCreatureEvent>,
) {
    for (entity, mut spawner, transform) in spawners.iter_mut() {
        if !spawner.active {
            continue;
        }

        spawner.spawn_interval.tick(time.delta());

        if spawner.spawn_interval.just_finished() && spawner.can_spawn() {
            spawn_events.write(SpawnCreatureEvent {
                spawner: entity,
                position: transform.translation,
                team: Team::Player,
            });
        }
    }
}

/// Tracks creature deaths and handles respawning
pub fn respawn_tracking_system(
    mut spawners: Query<&mut Spawner>,
    creatures: Query<Entity, With<SpawnedBy>>,
) {
    for mut spawner in spawners.iter_mut() {
        // Remove dead creatures from the spawned list
        spawner.spawned_creatures.retain(|&entity| {
            creatures.get(entity).is_ok()
        });
    }
}

//! Basic FloofByte - The starter defender creature
//!
//! A simple melee creature that attacks nearby enemies.

use bevy::prelude::*;
use ddd_core::{
    CombatStats, CreatureExperience, Health, Lifetime, MovementSpeed, OnFloor, Team,
};

use crate::{SpawnCreatureEvent, SpawnedBy, Spawner};

pub struct BasicFloofPlugin;

impl Plugin for BasicFloofPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<BasicFloof>()
            .add_systems(Update, (
                basic_floof_spawn_handler,
                basic_floof_ai_system,
            ));
    }
}

/// Marker component for Basic FloofBytes
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BasicFloof;

/// Bundle containing all components for a Basic FloofByte
#[derive(Bundle)]
pub struct BasicFloofBundle {
    pub marker: BasicFloof,
    pub health: Health,
    pub speed: MovementSpeed,
    pub combat_stats: CombatStats,
    pub experience: CreatureExperience,
    pub lifetime: Lifetime,
    pub team: Team,
    pub on_floor: OnFloor,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl BasicFloofBundle {
    pub fn new(position: Vec3, floor: usize) -> Self {
        Self {
            marker: BasicFloof,
            health: Health::new(50.0),
            speed: MovementSpeed(80.0),
            combat_stats: CombatStats {
                attack: 10.0,
                defense: 5.0,
                attack_speed: 1.0,
                attack_range: 40.0,
            },
            experience: CreatureExperience::default(),
            lifetime: Lifetime::new(120.0), // 2 minute lifespan
            team: Team::Player,
            on_floor: OnFloor(floor),
            sprite: Sprite {
                color: Color::srgb(0.4, 1.0, 0.4), // Green for player creatures
                custom_size: Some(Vec2::splat(24.0)),
                ..default()
            },
            transform: Transform::from_translation(position),
        }
    }
}

/// Handles spawn events for BasicFloof spawners
fn basic_floof_spawn_handler(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnCreatureEvent>,
    mut spawners: Query<(&mut Spawner, &OnFloor), With<BasicFloofSpawner>>,
) {
    for event in spawn_events.read() {
        if let Ok((mut spawner, on_floor)) = spawners.get_mut(event.spawner) {
            if spawner.can_spawn() {
                let entity = commands.spawn((
                    BasicFloofBundle::new(event.position, on_floor.0),
                    SpawnedBy(event.spawner),
                )).id();

                spawner.spawned_creatures.push(entity);
            }
        }
    }
}

/// Marker for spawners that create BasicFloofs
#[derive(Component, Default)]
pub struct BasicFloofSpawner;

/// Bundle for creating a BasicFloof spawner
#[derive(Bundle)]
pub struct BasicFloofSpawnerBundle {
    pub spawner: Spawner,
    pub marker: BasicFloofSpawner,
    pub on_floor: OnFloor,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl BasicFloofSpawnerBundle {
    pub fn new(position: Vec3, floor: usize) -> Self {
        Self {
            spawner: Spawner::new(3, 8.0, 15.0),
            marker: BasicFloofSpawner,
            on_floor: OnFloor(floor),
            sprite: Sprite {
                color: Color::srgb(0.2, 0.6, 0.2), // Darker green for spawner
                custom_size: Some(Vec2::splat(32.0)),
                ..default()
            },
            transform: Transform::from_translation(position),
        }
    }
}

/// Simple AI for BasicFloof - moves toward nearest enemy and attacks
fn basic_floof_ai_system(
    mut floofs: Query<(&mut Transform, &MovementSpeed, &CombatStats), With<BasicFloof>>,
    enemies: Query<&Transform, (With<Team>, Without<BasicFloof>)>,
    time: Res<Time>,
) {
    for (mut floof_transform, speed, combat_stats) in floofs.iter_mut() {
        // Find nearest enemy
        let mut nearest_dist = f32::MAX;
        let mut nearest_pos = None;

        for enemy_transform in enemies.iter() {
            let dist = floof_transform.translation.distance(enemy_transform.translation);
            if dist < nearest_dist {
                nearest_dist = dist;
                nearest_pos = Some(enemy_transform.translation);
            }
        }

        // Move toward nearest enemy if outside attack range
        if let Some(target_pos) = nearest_pos {
            if nearest_dist > combat_stats.attack_range {
                let direction = (target_pos - floof_transform.translation).normalize_or_zero();
                floof_transform.translation += direction * speed.0 * time.delta_secs();
            }
        }
    }
}

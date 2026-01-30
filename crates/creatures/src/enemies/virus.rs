//! Virus - Basic enemy creature
//!
//! A simple enemy that pathfinds toward the dungeon core.

use bevy::prelude::*;
use ddd_core::{
    AttackCooldown, CombatStats, DungeonCore, DungeonGrid, GridPosition, Health,
    MovementSpeed, OnFloor, PathAgent, Team, find_path,
};

pub struct VirusPlugin;

impl Plugin for VirusPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Virus>()
            .add_systems(Update, (
                virus_pathfinding_system,
                virus_movement_system,
            ).chain());
    }
}

/// Marker component for Virus enemies
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Virus;

/// Bundle containing all components for a Virus
#[derive(Bundle)]
pub struct VirusBundle {
    pub marker: Virus,
    pub health: Health,
    pub speed: MovementSpeed,
    pub combat_stats: CombatStats,
    pub attack_cooldown: AttackCooldown,
    pub team: Team,
    pub on_floor: OnFloor,
    pub path_agent: PathAgent,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl VirusBundle {
    pub fn new(position: Vec3, floor: usize) -> Self {
        let attack_speed = 1.2;
        Self {
            marker: Virus,
            health: Health::new(30.0),
            speed: MovementSpeed(60.0),
            combat_stats: CombatStats {
                attack: 8.0,
                defense: 2.0,
                attack_speed,
                attack_range: 30.0,
            },
            attack_cooldown: AttackCooldown::new(attack_speed),
            team: Team::Enemy,
            on_floor: OnFloor(floor),
            path_agent: PathAgent::new(),
            sprite: Sprite {
                color: Color::srgb(1.0, 0.2, 0.2), // Red for enemies
                custom_size: Some(Vec2::splat(20.0)),
                ..default()
            },
            transform: Transform::from_translation(position),
        }
    }
}

/// System that calculates paths for viruses toward the core
fn virus_pathfinding_system(
    mut viruses: Query<(&Transform, &OnFloor, &mut PathAgent), With<Virus>>,
    core: Query<(&Transform, &GridPosition), With<DungeonCore>>,
    dungeon: Res<DungeonGrid>,
) {
    let Ok((core_transform, core_grid_pos)) = core.single() else {
        return;
    };

    for (transform, on_floor, mut path_agent) in viruses.iter_mut() {
        // Only recalculate if path is empty or complete
        if !path_agent.is_complete() && !path_agent.path.is_empty() {
            continue;
        }

        // Get current grid position
        let world_pos = transform.translation.truncate();
        if let Some(current_grid) = dungeon.world_to_grid(world_pos) {
            // Find path to core
            if let Some(path) = find_path(&dungeon, current_grid, *core_grid_pos) {
                path_agent.set_path(path);
            }
        }
    }
}

/// System that moves viruses along their paths
fn virus_movement_system(
    mut viruses: Query<(&mut Transform, &MovementSpeed, &mut PathAgent), With<Virus>>,
    dungeon: Res<DungeonGrid>,
    time: Res<Time>,
) {
    for (mut transform, speed, mut path_agent) in viruses.iter_mut() {
        if let Some(waypoint) = path_agent.current_waypoint() {
            let target_world = dungeon.grid_to_world(waypoint);
            let current = transform.translation.truncate();
            let direction = (target_world - current).normalize_or_zero();

            // Move toward waypoint
            let movement = direction * speed.0 * time.delta_secs();
            transform.translation.x += movement.x;
            transform.translation.y += movement.y;

            // Check if we've reached the waypoint
            if current.distance(target_world) < 5.0 {
                path_agent.advance();
            }
        }
    }
}

/// Spawns a wave of viruses
pub fn spawn_virus_wave(
    commands: &mut Commands,
    count: usize,
    spawn_positions: &[Vec3],
    floor: usize,
) {
    for (i, &position) in spawn_positions.iter().enumerate().take(count) {
        commands.spawn(VirusBundle::new(position, floor));
    }
}

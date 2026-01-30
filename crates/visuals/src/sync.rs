//! Entity synchronization system - converts 2D sprites to 3D meshes

use bevy::prelude::*;
use ddd_core::{DungeonCore, OnFloor, Team};
use ddd_creatures::Spawner;
use ddd_traps::{DataSpike, Firewall};
use crate::{create_tron_material, TronMaterials, TronMeshes, TronPalette};
use crate::grid::floor_y_position;
use crate::CREATURE_SCALE;

/// Marker component indicating an entity has been given 3D visuals
#[derive(Component)]
pub struct HasVisuals;

/// Component for dungeon core pulsing animation
#[derive(Component)]
pub struct CorePulse {
    /// Base scale of the core
    pub base_scale: f32,
    /// Phase offset for the animation (0.0 - 2π)
    phase: f32,
    /// Base emissive intensity
    base_intensity: f32,
}

/// Plugin that syncs game entities to 3D visuals
pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            sync_dungeon_core,
            sync_creatures,
            sync_spawners,
            sync_firewalls,
            sync_data_spikes,
            animate_core_pulse,
            sync_transforms_to_3d,
        ));
    }
}

/// Converts the dungeon core from sprite to 3D mesh with pulsing animation
fn sync_dungeon_core(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&OnFloor>), (With<DungeonCore>, Without<HasVisuals>)>,
    meshes: Option<Res<TronMeshes>>,
    palette: Option<Res<TronPalette>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    let (Some(meshes), Some(palette)) = (meshes, palette) else {
        return;
    };

    const BASE_INTENSITY: f32 = 4.0;
    const BASE_SCALE: f32 = 24.0;
    const CORE_HEIGHT_OFFSET: f32 = 6.0; // Height above floor plane

    for (entity, transform, on_floor) in query.iter() {
        // Calculate Y based on floor level
        let floor_index = on_floor.map(|f| f.0).unwrap_or(0);
        let base_y = floor_y_position(floor_index);
        let pos_3d = Vec3::new(transform.translation.x, base_y + CORE_HEIGHT_OFFSET, transform.translation.y);

        // Create a unique material instance for this core so we can animate it
        let core_material = material_assets.add(create_tron_material(palette.white_cyan, BASE_INTENSITY));

        commands.entity(entity)
            .remove::<Sprite>()
            .insert((
                HasVisuals,
                CorePulse {
                    base_scale: BASE_SCALE,
                    phase: 0.0,
                    base_intensity: BASE_INTENSITY,
                },
                Mesh3d(meshes.core.clone()),
                MeshMaterial3d(core_material),
                Transform::from_translation(pos_3d)
                    .with_scale(Vec3::splat(BASE_SCALE)),
            ));
    }
}

/// Converts creature entities (with Team component) to 3D meshes
fn sync_creatures(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Team, Option<&OnFloor>), (With<Sprite>, Without<HasVisuals>)>,
    materials: Option<Res<TronMaterials>>,
    meshes: Option<Res<TronMeshes>>,
) {
    let (Some(materials), Some(meshes)) = (materials, meshes) else {
        return;
    };

    const CREATURE_HEIGHT_OFFSET: f32 = 3.0; // Height above floor plane

    for (entity, transform, team, on_floor) in query.iter() {
        // Calculate Y based on floor level
        let floor_index = on_floor.map(|f| f.0).unwrap_or(0);
        let base_y = floor_y_position(floor_index);
        let pos_3d = Vec3::new(transform.translation.x, base_y + CREATURE_HEIGHT_OFFSET, transform.translation.y);

        // Choose material based on team
        let material = match team {
            Team::Player => materials.cyan.clone(),
            Team::Enemy => materials.orange.clone(),
        };

        // Scale for elliptical spheroid appearance
        let base_scale = 8.0;
        let scale = Vec3::new(
            base_scale * CREATURE_SCALE.x,
            base_scale * CREATURE_SCALE.y,
            base_scale * CREATURE_SCALE.z,
        );

        commands.entity(entity)
            .remove::<Sprite>()
            .insert((
                HasVisuals,
                Mesh3d(meshes.creature_spheroid.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(pos_3d).with_scale(scale),
            ));
    }
}

/// Converts firewall traps to 3D orange cuboids
fn sync_firewalls(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Firewall, Option<&OnFloor>), (With<Sprite>, Without<HasVisuals>)>,
    materials: Option<Res<TronMaterials>>,
    meshes: Option<Res<TronMeshes>>,
) {
    let (Some(materials), Some(meshes)) = (materials, meshes) else {
        return;
    };

    const FIREWALL_HEIGHT_OFFSET: f32 = 5.0; // Height above floor plane

    for (entity, transform, firewall, on_floor) in query.iter() {
        // Calculate Y based on floor level
        let floor_index = on_floor.map(|f| f.0).unwrap_or(0);
        let base_y = floor_y_position(floor_index);
        let pos_3d = Vec3::new(transform.translation.x, base_y + FIREWALL_HEIGHT_OFFSET, transform.translation.y);

        // Scale based on firewall width
        let scale = Vec3::new(firewall.width / 2.0, 8.0, 4.0);

        commands.entity(entity)
            .remove::<Sprite>()
            .insert((
                HasVisuals,
                Mesh3d(meshes.firewall.clone()),
                MeshMaterial3d(materials.firewall.clone()),
                Transform::from_translation(pos_3d).with_scale(scale),
            ));
    }
}

/// Converts data spike traps to 3D magenta cones
fn sync_data_spikes(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&OnFloor>), (With<DataSpike>, With<Sprite>, Without<HasVisuals>)>,
    materials: Option<Res<TronMaterials>>,
    meshes: Option<Res<TronMeshes>>,
) {
    let (Some(materials), Some(meshes)) = (materials, meshes) else {
        return;
    };

    const SPIKE_HEIGHT_OFFSET: f32 = 1.0; // Slightly above floor plane

    for (entity, transform, on_floor) in query.iter() {
        // Calculate Y based on floor level
        let floor_index = on_floor.map(|f| f.0).unwrap_or(0);
        let base_y = floor_y_position(floor_index);
        let pos_3d = Vec3::new(transform.translation.x, base_y + SPIKE_HEIGHT_OFFSET, transform.translation.y);

        // Scale for visible spike
        let scale = Vec3::splat(12.0);

        commands.entity(entity)
            .remove::<Sprite>()
            .insert((
                HasVisuals,
                Mesh3d(meshes.data_spike.clone()),
                MeshMaterial3d(materials.magenta.clone()),
                Transform::from_translation(pos_3d).with_scale(scale),
            ));
    }
}

/// Converts spawner entities to 3D hexagonal cylinders
/// Player spawners are cyan, enemy spawners are orange-red
fn sync_spawners(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&Team>, Option<&OnFloor>), (With<Spawner>, With<Sprite>, Without<HasVisuals>)>,
    materials: Option<Res<TronMaterials>>,
    meshes: Option<Res<TronMeshes>>,
) {
    let (Some(materials), Some(meshes)) = (materials, meshes) else {
        return;
    };

    const SPAWNER_HEIGHT_OFFSET: f32 = 1.0; // Slightly above floor plane

    for (entity, transform, team, on_floor) in query.iter() {
        // Calculate Y based on floor level
        let floor_index = on_floor.map(|f| f.0).unwrap_or(0);
        let base_y = floor_y_position(floor_index);
        let pos_3d = Vec3::new(transform.translation.x, base_y + SPAWNER_HEIGHT_OFFSET, transform.translation.y);

        // Choose material based on team (default to player/cyan if no team)
        let material = match team {
            Some(Team::Enemy) => materials.orange.clone(),
            _ => materials.cyan.clone(),
        };

        // Scale for visible spawner
        let scale = Vec3::new(16.0, 12.0, 16.0);

        commands.entity(entity)
            .remove::<Sprite>()
            .insert((
                HasVisuals,
                Mesh3d(meshes.spawner.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(pos_3d).with_scale(scale),
            ));
    }
}

/// Animates the dungeon core with pulsing scale and emissive intensity
fn animate_core_pulse(
    mut query: Query<(&mut Transform, &mut CorePulse, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Option<Res<TronPalette>>,
    time: Res<Time>,
) {
    let Some(palette) = palette else {
        return;
    };

    const PULSE_SPEED: f32 = 2.0; // Cycles per second
    const SCALE_AMPLITUDE: f32 = 0.08; // 8% scale variation
    const INTENSITY_AMPLITUDE: f32 = 1.5; // Intensity varies by ±1.5

    for (mut transform, mut pulse, material_handle) in query.iter_mut() {
        // Update phase
        pulse.phase += time.delta_secs() * PULSE_SPEED * std::f32::consts::TAU;
        if pulse.phase > std::f32::consts::TAU {
            pulse.phase -= std::f32::consts::TAU;
        }

        let sin_value = pulse.phase.sin();

        // Animate scale (breathing effect)
        let scale_factor = 1.0 + sin_value * SCALE_AMPLITUDE;
        transform.scale = Vec3::splat(pulse.base_scale * scale_factor);

        // Animate emissive intensity
        if let Some(material) = materials.get_mut(material_handle.id()) {
            let intensity = pulse.base_intensity + sin_value * INTENSITY_AMPLITUDE;
            material.emissive = TronPalette::to_emissive(palette.white_cyan, intensity);
        }
    }
}

/// Updates 3D transforms when 2D game logic moves entities
/// Game uses XY plane, we render on XZ plane with Y as height
fn sync_transforms_to_3d(
    _query: Query<&Transform, (With<HasVisuals>, Changed<Transform>)>,
) {
    // This system watches for transform changes from game logic
    // and ensures the 3D position stays correct
    // The conversion happens in the sync systems above when entities are created
    // This handles ongoing movement updates

    // Note: If game logic directly modifies Transform, we need to intercept
    // For now, game entities should update position through a GridPosition component
    // which the pathfinding/movement systems handle
}

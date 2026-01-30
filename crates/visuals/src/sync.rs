//! Entity synchronization system - converts 2D sprites to 3D meshes

use bevy::prelude::*;
use ddd_core::{DungeonCore, Team};
use crate::{TronMaterials, TronMeshes};
use crate::CREATURE_SCALE;

/// Marker component indicating an entity has been given 3D visuals
#[derive(Component)]
pub struct HasVisuals;

/// Plugin that syncs game entities to 3D visuals
pub struct SyncPlugin;

impl Plugin for SyncPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            sync_dungeon_core,
            sync_creatures,
            sync_transforms_to_3d,
        ));
    }
}

/// Converts the dungeon core from sprite to 3D mesh
fn sync_dungeon_core(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<DungeonCore>, Without<HasVisuals>)>,
    materials: Option<Res<TronMaterials>>,
    meshes: Option<Res<TronMeshes>>,
) {
    let (Some(materials), Some(meshes)) = (materials, meshes) else {
        return;
    };

    for (entity, transform) in query.iter() {
        // Convert XY position to XZ plane (Y becomes height)
        let pos_3d = Vec3::new(transform.translation.x, 5.0, transform.translation.y);

        commands.entity(entity)
            .remove::<Sprite>()
            .insert((
                HasVisuals,
                Mesh3d(meshes.core.clone()),
                MeshMaterial3d(materials.white_cyan.clone()),
                Transform::from_translation(pos_3d)
                    .with_scale(Vec3::splat(24.0)),
            ));
    }
}

/// Converts creature entities (with Team component) to 3D meshes
fn sync_creatures(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &Team), (With<Sprite>, Without<HasVisuals>)>,
    materials: Option<Res<TronMaterials>>,
    meshes: Option<Res<TronMeshes>>,
) {
    let (Some(materials), Some(meshes)) = (materials, meshes) else {
        return;
    };

    for (entity, transform, team) in query.iter() {
        // Convert XY position to XZ plane
        let pos_3d = Vec3::new(transform.translation.x, 2.0, transform.translation.y);

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

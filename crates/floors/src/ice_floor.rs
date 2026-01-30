//! Ice Floor - Slows enemies and boosts ice damage
//!
//! A floor type that slows down enemies and enhances ice-based abilities.

use bevy::prelude::*;
use ddd_core::{DamageType, MovementSpeed, OnFloor, Team};

use crate::{DamageModifier, FloorTypeMarker, SpeedModifier};

pub struct IceFloorPlugin;

impl Plugin for IceFloorPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<IceFloor>()
            .add_systems(Update, ice_floor_slow_system);
    }
}

/// Marker component for ice floors
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct IceFloor {
    pub slow_percentage: f32,
}

impl IceFloor {
    pub fn new() -> Self {
        Self {
            slow_percentage: 0.30, // 30% slow
        }
    }
}

/// Bundle for creating an ice floor
#[derive(Bundle)]
pub struct IceFloorBundle {
    pub ice_floor: IceFloor,
    pub floor_marker: FloorTypeMarker,
    pub damage_modifier: DamageModifier,
    pub speed_modifier: SpeedModifier,
}

impl IceFloorBundle {
    pub fn new(floor_index: usize) -> Self {
        let ice_floor = IceFloor::new();
        Self {
            speed_modifier: SpeedModifier::new(1.0 - ice_floor.slow_percentage).enemies_only(),
            damage_modifier: DamageModifier::typed(1.25, DamageType::Ice), // +25% ice damage
            floor_marker: FloorTypeMarker(floor_index),
            ice_floor,
        }
    }
}

/// Tracks entities affected by ice slow (to avoid stacking)
#[derive(Component)]
pub struct IceSlowed {
    pub original_speed: f32,
}

/// System that applies slow effect to enemies on ice floors
fn ice_floor_slow_system(
    ice_floors: Query<(&FloorTypeMarker, &IceFloor)>,
    mut entities: Query<(Entity, &OnFloor, &Team, &mut MovementSpeed, Option<&IceSlowed>)>,
    mut commands: Commands,
) {
    // Build set of floor indices that are ice floors
    let ice_floor_indices: std::collections::HashSet<usize> = ice_floors
        .iter()
        .map(|(marker, _)| marker.0)
        .collect();

    // Get slow values for ice floors
    let ice_slow_values: std::collections::HashMap<usize, f32> = ice_floors
        .iter()
        .map(|(marker, ice)| (marker.0, ice.slow_percentage))
        .collect();

    for (entity, on_floor, team, mut speed, ice_slowed) in entities.iter_mut() {
        let on_ice_floor = ice_floor_indices.contains(&on_floor.0);

        match (on_ice_floor, ice_slowed, team) {
            // Enemy on ice floor, not yet slowed -> apply slow
            (true, None, Team::Enemy) => {
                if let Some(&slow_pct) = ice_slow_values.get(&on_floor.0) {
                    let original = speed.0;
                    speed.0 *= 1.0 - slow_pct;
                    commands.entity(entity).insert(IceSlowed { original_speed: original });
                }
            }
            // Entity left ice floor but still has slow -> remove slow
            (false, Some(slowed), _) => {
                speed.0 = slowed.original_speed;
                commands.entity(entity).remove::<IceSlowed>();
            }
            // Already slowed on ice floor, or ally on ice floor -> do nothing
            _ => {}
        }
    }
}

/// Adjacent floor bonus - ice floors give +10% ice damage to adjacent floors
#[derive(Component)]
pub struct IceFloorAdjacencyBonus {
    pub ice_damage_bonus: f32,
}

impl Default for IceFloorAdjacencyBonus {
    fn default() -> Self {
        Self {
            ice_damage_bonus: 0.10,
        }
    }
}

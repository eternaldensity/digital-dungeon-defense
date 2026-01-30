//! Fire Floor - Deals damage over time and boosts fire damage
//!
//! A floor type that burns enemies and enhances fire-based abilities.

use bevy::prelude::*;
use ddd_core::{DamageEvent, DamageType, OnFloor, Team};

use crate::{DamageModifier, FloorDamageOverTime, FloorTypeMarker};

pub struct FireFloorPlugin;

impl Plugin for FireFloorPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<FireFloor>()
            .add_systems(Update, fire_floor_dot_system);
    }
}

/// Marker component for fire floors
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct FireFloor {
    pub burn_damage: f32,
    pub burn_interval: f32,
}

impl FireFloor {
    pub fn new() -> Self {
        Self {
            burn_damage: 5.0,
            burn_interval: 1.0,
        }
    }
}

/// Bundle for creating a fire floor
#[derive(Bundle)]
pub struct FireFloorBundle {
    pub fire_floor: FireFloor,
    pub floor_marker: FloorTypeMarker,
    pub damage_modifier: DamageModifier,
    pub dot: FloorDamageOverTime,
}

impl FireFloorBundle {
    pub fn new(floor_index: usize) -> Self {
        let fire_floor = FireFloor::new();
        Self {
            dot: FloorDamageOverTime::new(
                fire_floor.burn_damage,
                DamageType::Fire,
                fire_floor.burn_interval,
            ),
            damage_modifier: DamageModifier::typed(1.25, DamageType::Fire), // +25% fire damage
            floor_marker: FloorTypeMarker(floor_index),
            fire_floor,
        }
    }
}

/// System that applies burn damage to enemies on fire floors
fn fire_floor_dot_system(
    mut fire_floors: Query<(&FloorTypeMarker, &mut FloorDamageOverTime), With<FireFloor>>,
    entities_on_floors: Query<(Entity, &OnFloor, &Team)>,
    mut damage_events: EventWriter<DamageEvent>,
    time: Res<Time>,
) {
    for (floor_marker, mut dot) in fire_floors.iter_mut() {
        dot.tick_timer.tick(time.delta());

        if dot.tick_timer.just_finished() {
            // Damage all enemies on this floor
            for (entity, on_floor, team) in entities_on_floors.iter() {
                if on_floor.0 != floor_marker.0 {
                    continue;
                }

                // Check if we should affect this entity
                let should_damage = match team {
                    Team::Enemy => dot.affects_enemies,
                    Team::Player => dot.affects_allies,
                };

                if should_damage {
                    damage_events.send(DamageEvent {
                        target: entity,
                        source: None,
                        amount: dot.damage_per_tick,
                        damage_type: dot.damage_type,
                    });
                }
            }
        }
    }
}

/// Adjacent floor bonus - fire floors give +10% fire damage to adjacent floors
#[derive(Component)]
pub struct FireFloorAdjacencyBonus {
    pub fire_damage_bonus: f32,
}

impl Default for FireFloorAdjacencyBonus {
    fn default() -> Self {
        Self {
            fire_damage_bonus: 0.10,
        }
    }
}

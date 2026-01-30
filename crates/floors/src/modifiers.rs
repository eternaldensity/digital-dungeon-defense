//! Floor modifier system - shared components for floor effects

use bevy::prelude::*;
use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use ddd_core::{DamageEvent, DamageType, MovementSpeed, OnFloor, Team};

pub struct ModifiersPlugin;

impl Plugin for ModifiersPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<DamageModifier>()
            .register_type::<SpeedModifier>()
            .add_message::<ModifiedDamageEvent>()
            .add_systems(Update, (
                apply_floor_damage_modifiers,
                apply_speed_modifiers,
            ));
    }
}

/// Modifies damage dealt on this floor
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DamageModifier {
    pub multiplier: f32,
    /// If Some, only applies to this damage type. If None, applies to all.
    pub damage_type: Option<DamageType>,
    /// Whether this affects allies, enemies, or both
    pub affects_allies: bool,
    pub affects_enemies: bool,
}

impl DamageModifier {
    pub fn all_damage(multiplier: f32) -> Self {
        Self {
            multiplier,
            damage_type: None,
            affects_allies: true,
            affects_enemies: true,
        }
    }

    pub fn typed(multiplier: f32, damage_type: DamageType) -> Self {
        Self {
            multiplier,
            damage_type: Some(damage_type),
            affects_allies: true,
            affects_enemies: true,
        }
    }

    pub fn enemies_only(mut self) -> Self {
        self.affects_allies = false;
        self
    }

    pub fn allies_only(mut self) -> Self {
        self.affects_enemies = false;
        self
    }
}

/// Modifies movement speed on this floor
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct SpeedModifier {
    pub multiplier: f32,
    pub affects_allies: bool,
    pub affects_enemies: bool,
}

impl SpeedModifier {
    pub fn new(multiplier: f32) -> Self {
        Self {
            multiplier,
            affects_allies: true,
            affects_enemies: true,
        }
    }

    pub fn enemies_only(mut self) -> Self {
        self.affects_allies = false;
        self
    }

    pub fn allies_only(mut self) -> Self {
        self.affects_enemies = false;
        self
    }
}

/// Damage over time effect from floors
#[derive(Component)]
pub struct FloorDamageOverTime {
    pub damage_per_tick: f32,
    pub damage_type: DamageType,
    pub tick_timer: Timer,
    pub affects_allies: bool,
    pub affects_enemies: bool,
}

impl FloorDamageOverTime {
    pub fn new(damage_per_tick: f32, damage_type: DamageType, tick_interval: f32) -> Self {
        Self {
            damage_per_tick,
            damage_type,
            tick_timer: Timer::from_seconds(tick_interval, TimerMode::Repeating),
            affects_allies: false,
            affects_enemies: true,
        }
    }
}

/// Links an entity to the floor it's associated with (for floor type entities)
#[derive(Component)]
pub struct FloorTypeMarker(pub usize);

/// Tracks the base movement speed before modifiers
#[derive(Component)]
pub struct BaseMovementSpeed(pub f32);

/// Tracks which floor's modifier is currently applied (to detect floor changes)
#[derive(Component)]
pub struct CurrentFloorModifier(pub Option<usize>);

/// Calculates the damage multiplier for a given floor and damage type
pub fn get_floor_damage_multiplier(
    floor_idx: usize,
    damage_type: DamageType,
    floor_modifiers: &[(usize, DamageModifier)],
) -> f32 {
    let mut multiplier = 1.0;

    for (modifier_floor, modifier) in floor_modifiers {
        if *modifier_floor != floor_idx {
            continue;
        }

        // Check if modifier applies to this damage type
        let type_matches = match modifier.damage_type {
            Some(dt) => dt == damage_type,
            None => true, // None means applies to all types
        };

        if type_matches {
            multiplier *= modifier.multiplier;
        }
    }

    multiplier
}

/// Applies speed modifiers from floors
fn apply_speed_modifiers(
    floors: Query<(&FloorTypeMarker, &SpeedModifier)>,
    mut entities: Query<(
        &OnFloor,
        &mut MovementSpeed,
        Option<&BaseMovementSpeed>,
        Option<&mut CurrentFloorModifier>,
    )>,
) {
    // Create a map of floor -> modifier
    let floor_modifiers: std::collections::HashMap<usize, f32> = floors
        .iter()
        .map(|(marker, modifier)| (marker.0, modifier.multiplier))
        .collect();

    for (on_floor, mut speed, base_speed, current_modifier) in entities.iter_mut() {
        let floor_idx = on_floor.0;
        let modifier = floor_modifiers.get(&floor_idx).copied();

        // Check if we need to update the speed
        let needs_update = match &current_modifier {
            Some(cm) => cm.0 != modifier.map(|_| floor_idx),
            None => modifier.is_some(),
        };

        if !needs_update {
            continue;
        }

        // Get or calculate base speed
        let base = match base_speed {
            Some(bs) => bs.0,
            None => speed.0, // Current speed is base if not tracked yet
        };

        // Apply modifier
        let new_speed = match modifier {
            Some(multiplier) => base * multiplier,
            None => base,
        };

        speed.0 = new_speed;
    }
}

/// Modified damage event that includes floor modifiers applied
#[derive(Message)]
pub struct ModifiedDamageEvent {
    pub target: Entity,
    pub source: Option<Entity>,
    pub original_amount: f32,
    pub modified_amount: f32,
    pub damage_type: DamageType,
    pub floor_multiplier: f32,
}

/// System that reads damage events and applies floor modifiers
fn apply_floor_damage_modifiers(
    mut damage_events: MessageReader<DamageEvent>,
    mut modified_events: MessageWriter<ModifiedDamageEvent>,
    floors: Query<(&FloorTypeMarker, &DamageModifier)>,
    targets: Query<(&OnFloor, Option<&Team>)>,
) {
    // Collect floor modifiers
    let floor_modifiers: Vec<(usize, &DamageModifier)> = floors
        .iter()
        .map(|(marker, modifier)| (marker.0, modifier))
        .collect();

    for event in damage_events.read() {
        let mut multiplier = 1.0;

        // Get target's floor
        if let Ok((on_floor, target_team)) = targets.get(event.target) {
            let floor_idx = on_floor.0;
            let is_enemy = target_team.map_or(false, |t| *t == Team::Enemy);

            // Apply matching floor modifiers
            for (modifier_floor, modifier) in &floor_modifiers {
                if *modifier_floor != floor_idx {
                    continue;
                }

                // Check team restrictions
                let team_applies = (is_enemy && modifier.affects_enemies)
                    || (!is_enemy && modifier.affects_allies);

                if !team_applies {
                    continue;
                }

                // Check damage type
                let type_matches = match modifier.damage_type {
                    Some(dt) => dt == event.damage_type,
                    None => true,
                };

                if type_matches {
                    multiplier *= modifier.multiplier;
                }
            }
        }

        modified_events.write(ModifiedDamageEvent {
            target: event.target,
            source: event.source,
            original_amount: event.amount,
            modified_amount: event.amount * multiplier,
            damage_type: event.damage_type,
            floor_multiplier: multiplier,
        });
    }
}

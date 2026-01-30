//! Floor modifier system - shared components for floor effects

use bevy::prelude::*;
use ddd_core::{DamageType, MovementSpeed, OnFloor};

pub struct ModifiersPlugin;

impl Plugin for ModifiersPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<DamageModifier>()
            .register_type::<SpeedModifier>()
            .add_systems(Update, (
                apply_damage_modifiers,
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

/// Applies damage modifiers from floors
/// Note: This is a simplified version - a full implementation would
/// intercept DamageEvents and modify them based on floor modifiers
fn apply_damage_modifiers(
    // Floor modifiers would be queried and applied to damage events
    // This is a placeholder for the full implementation
) {
    // TODO: Implement damage modifier application
    // Would need to intercept damage events and modify amounts
}

/// Applies speed modifiers from floors
fn apply_speed_modifiers(
    floors: Query<(&FloorTypeMarker, &SpeedModifier)>,
    mut entities: Query<(&OnFloor, &mut MovementSpeed)>,
) {
    // Create a map of floor -> modifier
    let floor_modifiers: std::collections::HashMap<usize, f32> = floors
        .iter()
        .map(|(marker, modifier)| (marker.0, modifier.multiplier))
        .collect();

    for (on_floor, mut speed) in entities.iter_mut() {
        if let Some(&multiplier) = floor_modifiers.get(&on_floor.0) {
            // Note: This applies the modifier every frame which isn't ideal
            // A proper implementation would track base speed separately
            // This is simplified for the prototype
            speed.0 *= multiplier;
        }
    }
}

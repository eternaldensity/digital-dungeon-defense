//! Data Spike trap - High damage single-target trap
//!
//! A trap that deals heavy damage to a single enemy, then needs manual reset.

use bevy::prelude::*;
use ddd_core::{DamageEvent, DamageType, OnFloor, Team};

use crate::{ManualReset, Trap, TrapTriggeredEvent, TriggerZone};

pub struct DataSpikePlugin;

impl Plugin for DataSpikePlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<DataSpike>()
            .add_systems(Update, (
                data_spike_detection_system,
                data_spike_damage_system,
            ).chain());
    }
}

/// Data Spike trap component
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct DataSpike {
    pub damage: f32,
}

impl DataSpike {
    pub fn new() -> Self {
        Self {
            damage: 50.0, // High single-target damage
        }
    }
}

/// Bundle for creating a data spike trap
#[derive(Bundle)]
pub struct DataSpikeBundle {
    pub data_spike: DataSpike,
    pub trap: Trap,
    pub manual_reset: ManualReset,
    pub trigger_zone: TriggerZone,
    pub on_floor: OnFloor,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl DataSpikeBundle {
    pub fn new(position: Vec3, floor: usize) -> Self {
        Self {
            data_spike: DataSpike::new(),
            trap: Trap::new(0.0, 15.0, 35.0), // No cooldown, just manual reset
            manual_reset: ManualReset,
            trigger_zone: TriggerZone::new(20.0),
            on_floor: OnFloor(floor),
            sprite: Sprite {
                color: Color::srgb(0.8, 0.0, 0.8), // Purple
                custom_size: Some(Vec2::splat(24.0)),
                ..default()
            },
            transform: Transform::from_translation(position),
        }
    }
}

/// Detects enemies stepping on the data spike
fn data_spike_detection_system(
    mut spikes: Query<(
        Entity,
        &Transform,
        &TriggerZone,
        &OnFloor,
        &mut Trap,
    ), With<DataSpike>>,
    enemies: Query<(Entity, &Transform, &OnFloor, &Team)>,
    mut trigger_events: EventWriter<TrapTriggeredEvent>,
) {
    for (spike_entity, spike_transform, trigger_zone, spike_floor, mut trap) in spikes.iter_mut() {
        if !trap.can_trigger() {
            continue;
        }

        for (enemy_entity, enemy_transform, enemy_floor, team) in enemies.iter() {
            // Must be on same floor and be an enemy
            if spike_floor.0 != enemy_floor.0 || *team != Team::Enemy {
                continue;
            }

            // Check if in trigger zone
            let distance = spike_transform.translation.distance(enemy_transform.translation);
            if distance <= trigger_zone.radius {
                // Trigger - only hits one enemy then deactivates
                trap.trigger();
                trap.active = false; // Needs manual reset

                trigger_events.send(TrapTriggeredEvent {
                    trap: spike_entity,
                    triggered_by: enemy_entity,
                });

                break; // Only trigger on first enemy
            }
        }
    }
}

/// Applies damage when data spike is triggered
fn data_spike_damage_system(
    mut trigger_events: EventReader<TrapTriggeredEvent>,
    spikes: Query<&DataSpike>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for event in trigger_events.read() {
        if let Ok(spike) = spikes.get(event.trap) {
            damage_events.send(DamageEvent {
                target: event.triggered_by,
                source: Some(event.trap),
                amount: spike.damage,
                damage_type: DamageType::Viral, // Hacker-themed damage
            });
        }
    }
}

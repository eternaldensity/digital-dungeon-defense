//! Firewall trap - Damages enemies that pass through
//!
//! A barrier that deals fire damage to enemies crossing it.

use bevy::prelude::*;
use bevy::ecs::message::{MessageReader, MessageWriter};
use ddd_core::{DamageEvent, DamageType, OnFloor, Team};

use crate::{Trap, TrapTriggeredEvent, TriggerZone};

pub struct FirewallPlugin;

impl Plugin for FirewallPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Firewall>()
            .add_systems(Update, (
                firewall_detection_system,
                firewall_damage_system,
            ).chain());
    }
}

/// Firewall trap component
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Firewall {
    pub damage: f32,
    pub width: f32,
}

impl Firewall {
    pub fn new() -> Self {
        Self {
            damage: 15.0,
            width: 64.0,
        }
    }
}

/// Bundle for creating a firewall trap
#[derive(Bundle)]
pub struct FirewallBundle {
    pub firewall: Firewall,
    pub trap: Trap,
    pub trigger_zone: TriggerZone,
    pub on_floor: OnFloor,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl FirewallBundle {
    pub fn new(position: Vec3, floor: usize) -> Self {
        let firewall = Firewall::new();
        Self {
            trap: Trap::new(2.0, 8.0, 20.0),
            trigger_zone: TriggerZone::new(firewall.width / 2.0),
            on_floor: OnFloor(floor),
            sprite: Sprite {
                color: Color::srgba(1.0, 0.3, 0.0, 0.7), // Semi-transparent orange
                custom_size: Some(Vec2::new(firewall.width, 16.0)),
                ..default()
            },
            transform: Transform::from_translation(position),
            firewall,
        }
    }
}

/// Tracks which enemies have already been hit by this firewall (to prevent multi-hit)
#[derive(Component, Default)]
pub struct FirewallHitList {
    pub hit_entities: Vec<Entity>,
}

/// Detects enemies entering the firewall
fn firewall_detection_system(
    mut firewalls: Query<(
        Entity,
        &Transform,
        &TriggerZone,
        &OnFloor,
        &mut Trap,
        Option<&mut FirewallHitList>,
    ), With<Firewall>>,
    enemies: Query<(Entity, &Transform, &OnFloor), With<Team>>,
    teams: Query<&Team>,
    mut commands: Commands,
    mut trigger_events: MessageWriter<TrapTriggeredEvent>,
) {
    for (firewall_entity, fw_transform, trigger_zone, fw_floor, mut trap, hit_list) in firewalls.iter_mut() {
        if !trap.can_trigger() {
            continue;
        }

        for (enemy_entity, enemy_transform, enemy_floor) in enemies.iter() {
            // Must be on same floor
            if fw_floor.0 != enemy_floor.0 {
                continue;
            }

            // Must be an enemy
            if let Ok(team) = teams.get(enemy_entity) {
                if *team != Team::Enemy {
                    continue;
                }
            } else {
                continue;
            }

            // Check if in trigger zone
            let distance = fw_transform.translation.distance(enemy_transform.translation);
            if distance > trigger_zone.radius {
                continue;
            }

            // Check if already hit
            if let Some(ref list) = hit_list {
                if list.hit_entities.contains(&enemy_entity) {
                    continue;
                }
            }

            // Trigger the trap
            trap.trigger();
            trigger_events.write(TrapTriggeredEvent {
                trap: firewall_entity,
                triggered_by: enemy_entity,
            });

            // Add to hit list
            if hit_list.is_none() {
                commands.entity(firewall_entity).insert(FirewallHitList {
                    hit_entities: vec![enemy_entity],
                });
            }
        }
    }
}

/// Applies damage when firewall is triggered
fn firewall_damage_system(
    mut trigger_events: MessageReader<TrapTriggeredEvent>,
    firewalls: Query<&Firewall>,
    mut damage_events: MessageWriter<DamageEvent>,
) {
    for event in trigger_events.read() {
        if let Ok(firewall) = firewalls.get(event.trap) {
            damage_events.write(DamageEvent {
                target: event.triggered_by,
                source: Some(event.trap),
                amount: firewall.damage,
                damage_type: DamageType::Fire,
            });
        }
    }
}

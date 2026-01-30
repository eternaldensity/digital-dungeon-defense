//! Heal ability - Restore health to allies
//!
//! Entities with the Heal component can restore health to nearby allies.
//! Completely self-contained - all logic lives in this file.

use bevy::prelude::*;
use ddd_core::{AccumulatedTags, CombatTag, Health, Team};

use crate::{AbilityCooldown, AreaOfEffect, TargetsAllies};

pub struct HealPlugin;

impl Plugin for HealPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Heal>()
            .add_event::<HealEvent>()
            .add_systems(Update, (
                heal_cooldown_system,
                heal_targeting_system,
                apply_healing_system,
            ).chain());
    }
}

/// Heal ability component - attach to any entity to grant this ability
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Heal {
    pub amount: f32,
    pub range: f32,
    pub targets_self: bool,
}

impl Default for Heal {
    fn default() -> Self {
        Self {
            amount: 20.0,
            range: 100.0,
            targets_self: true,
        }
    }
}

/// Bundle to easily add healing capability to an entity
#[derive(Bundle)]
pub struct HealBundle {
    pub heal: Heal,
    pub cooldown: AbilityCooldown,
    pub targets_allies: TargetsAllies,
    pub aoe: AreaOfEffect,
}

impl Default for HealBundle {
    fn default() -> Self {
        let heal = Heal::default();
        Self {
            cooldown: AbilityCooldown::new(5.0),
            aoe: AreaOfEffect { radius: heal.range },
            heal,
            targets_allies: TargetsAllies,
        }
    }
}

/// Event fired when healing is applied
#[derive(Event)]
pub struct HealEvent {
    pub healer: Entity,
    pub target: Entity,
    pub amount: f32,
}

/// Tick cooldowns for heal abilities
fn heal_cooldown_system(
    mut query: Query<&mut AbilityCooldown, With<Heal>>,
    time: Res<Time>,
) {
    for mut cooldown in query.iter_mut() {
        cooldown.tick(time.delta());
    }
}

/// Find allies that need healing and heal them
fn heal_targeting_system(
    mut healers: Query<(
        Entity,
        &Transform,
        &Heal,
        &mut AbilityCooldown,
        &Team,
    )>,
    allies: Query<(Entity, &Transform, &Health, &Team)>,
    mut heal_events: EventWriter<HealEvent>,
) {
    for (healer_entity, healer_transform, heal, mut cooldown, healer_team) in healers.iter_mut() {
        if !cooldown.ready {
            continue;
        }

        // Find ally with lowest health percentage in range
        let mut best_target: Option<(Entity, f32)> = None;

        for (ally_entity, ally_transform, health, ally_team) in allies.iter() {
            // Only heal same team
            if healer_team != ally_team {
                continue;
            }

            // Skip self if not allowed
            if !heal.targets_self && ally_entity == healer_entity {
                continue;
            }

            // Skip if at full health
            if health.current >= health.maximum {
                continue;
            }

            let distance = healer_transform.translation.distance(ally_transform.translation);
            if distance <= heal.range {
                let health_fraction = health.fraction();
                if best_target.is_none() || health_fraction < best_target.unwrap().1 {
                    best_target = Some((ally_entity, health_fraction));
                }
            }
        }

        // Heal the most injured ally
        if let Some((target, _)) = best_target {
            cooldown.trigger();

            heal_events.send(HealEvent {
                healer: healer_entity,
                target,
                amount: heal.amount,
            });
        }
    }
}

/// Apply healing from HealEvents
fn apply_healing_system(
    mut heal_events: EventReader<HealEvent>,
    mut targets: Query<&mut Health>,
    mut healer_tags: Query<&mut AccumulatedTags>,
) {
    for event in heal_events.read() {
        if let Ok(mut health) = targets.get_mut(event.target) {
            health.heal(event.amount);
        }

        // Track that healer healed (for research unlocks)
        if let Ok(mut tags) = healer_tags.get_mut(event.healer) {
            tags.add_tag(CombatTag::Healed);
        }
    }
}

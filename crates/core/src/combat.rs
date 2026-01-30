//! Combat-related components and systems

use bevy::prelude::*;
use bevy::ecs::message::{MessageReader, MessageWriter};
use std::collections::HashMap;

use crate::{DamageEvent, DamageType, Health, PlayerResources, Team};

/// Combat statistics for creatures
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CombatStats {
    pub attack: f32,
    pub defense: f32,
    pub attack_speed: f32,
    pub attack_range: f32,
}

impl CombatStats {
    pub fn new(attack: f32, defense: f32) -> Self {
        Self {
            attack,
            defense,
            attack_speed: 1.0,
            attack_range: 50.0,
        }
    }
}

/// Movement speed component
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementSpeed(pub f32);

/// Lifetime component - entity despawns when timer finishes
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Lifetime {
    pub timer: Timer,
}

impl Default for Lifetime {
    fn default() -> Self {
        Self::new(60.0)
    }
}

impl Lifetime {
    pub fn new(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Once),
        }
    }
}

/// System that ticks lifetime timers and despawns expired entities
pub fn lifetime_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Experience tracking for individual creatures
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct CreatureExperience {
    pub total_xp: f32,
    pub combat_encounters: u32,
    pub waves_survived: u32,
}

impl CreatureExperience {
    pub fn add_combat_xp(&mut self, amount: f32) {
        self.total_xp += amount;
        self.combat_encounters += 1;
    }

    pub fn survive_wave(&mut self) {
        self.waves_survived += 1;
        self.total_xp += 10.0;
    }
}

/// Combat tags accumulated by creatures - used for research unlocks
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
pub enum CombatTag {
    DealtFireDamage,
    DealtIceDamage,
    DealtElectricDamage,
    DealtPhysicalDamage,
    TookDamage,
    SurvivedWave,
    KilledEnemy,
    KilledHacker,
    Healed,
    // Add new tags freely - systems check for specific tags, not exhaustive lists
}

/// Tracks accumulated combat tags for research unlocks
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct AccumulatedTags {
    pub tags: HashMap<CombatTag, u32>,
}

impl AccumulatedTags {
    pub fn add_tag(&mut self, tag: CombatTag) {
        *self.tags.entry(tag).or_insert(0) += 1;
    }

    pub fn get_count(&self, tag: CombatTag) -> u32 {
        *self.tags.get(&tag).unwrap_or(&0)
    }

    pub fn has_at_least(&self, tag: CombatTag, count: u32) -> bool {
        self.get_count(tag) >= count
    }
}

/// Marker for entities that want to attack (used by ability systems)
#[derive(Component)]
pub struct WantsToAttack {
    pub target: Entity,
}

/// Marker for entities currently in combat
#[derive(Component)]
pub struct InCombat;

/// Collects floof when enemies are damaged
pub fn collect_floof_system(
    mut damage_events: MessageReader<DamageEvent>,
    mut resources: ResMut<PlayerResources>,
    enemies: Query<&Team>,
) {
    for event in damage_events.read() {
        // Only collect floof from damaging enemies
        if let Ok(team) = enemies.get(event.target) {
            if *team == Team::Enemy {
                // Floof gained is proportional to damage dealt
                let floof_amount = event.amount * 0.1;
                resources.add_floof(floof_amount);
            }
        }
    }
}

/// Cooldown timer for auto-attacks
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct AttackCooldown {
    pub timer: Timer,
}

impl AttackCooldown {
    pub fn new(attack_speed: f32) -> Self {
        let cooldown_secs = if attack_speed > 0.0 {
            1.0 / attack_speed
        } else {
            1.0
        };
        Self {
            timer: Timer::from_seconds(cooldown_secs, TimerMode::Once),
        }
    }

    pub fn ready(&self) -> bool {
        self.timer.is_finished()
    }

    pub fn reset(&mut self) {
        self.timer.reset();
    }
}

/// Auto-attack system - creatures attack enemies in range when cooldown is ready
pub fn auto_attack_system(
    mut attackers: Query<(Entity, &Transform, &CombatStats, &Team, &mut AttackCooldown)>,
    targets: Query<(Entity, &Transform, &Team), With<Health>>,
    mut damage_events: MessageWriter<DamageEvent>,
    time: Res<Time>,
) {
    for (attacker_entity, attacker_transform, combat_stats, attacker_team, mut cooldown) in attackers.iter_mut() {
        cooldown.timer.tick(time.delta());

        if !cooldown.ready() {
            continue;
        }

        // Find nearest enemy in range
        let mut best_target: Option<Entity> = None;
        let mut best_distance = f32::MAX;

        for (target_entity, target_transform, target_team) in targets.iter() {
            // Skip same team
            if attacker_team == target_team {
                continue;
            }

            let distance = attacker_transform
                .translation
                .distance(target_transform.translation);

            if distance <= combat_stats.attack_range && distance < best_distance {
                best_distance = distance;
                best_target = Some(target_entity);
            }
        }

        // Attack if we found a target
        if let Some(target) = best_target {
            damage_events.write(DamageEvent {
                target,
                source: Some(attacker_entity),
                amount: combat_stats.attack,
                damage_type: DamageType::Physical,
            });
            cooldown.reset();
        }
    }
}

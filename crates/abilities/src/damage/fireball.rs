//! Fireball ability - Ranged fire damage
//!
//! Entities with the Fireball component can launch fireballs at enemies.
//! This is a completely self-contained ability - all logic lives here.

use bevy::prelude::*;
use ddd_core::{AccumulatedTags, CombatTag, DamageEvent, DamageType, Team};

use crate::{AbilityCooldown, AreaOfEffect, DealsDamage, TargetsEnemies};

pub struct FireballPlugin;

impl Plugin for FireballPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Fireball>()
            .add_event::<FireballCastEvent>()
            .add_systems(Update, (
                fireball_cooldown_system,
                fireball_targeting_system,
                fireball_projectile_system,
                fireball_impact_system,
            ).chain());
    }
}

/// Fireball ability component - attach to any entity to grant this ability
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Fireball {
    pub damage: f32,
    pub radius: f32,
    pub range: f32,
    pub projectile_speed: f32,
}

impl Default for Fireball {
    fn default() -> Self {
        Self {
            damage: 25.0,
            radius: 40.0,
            range: 200.0,
            projectile_speed: 300.0,
        }
    }
}

/// Bundle to easily add fireball capability to an entity
#[derive(Bundle)]
pub struct FireballBundle {
    pub fireball: Fireball,
    pub cooldown: AbilityCooldown,
    pub deals_damage: DealsDamage,
    pub targets_enemies: TargetsEnemies,
    pub aoe: AreaOfEffect,
}

impl Default for FireballBundle {
    fn default() -> Self {
        let fireball = Fireball::default();
        Self {
            cooldown: AbilityCooldown::new(3.0),
            deals_damage: DealsDamage {
                base_damage: fireball.damage,
                damage_type: DamageType::Fire,
            },
            aoe: AreaOfEffect { radius: fireball.radius },
            fireball,
            targets_enemies: TargetsEnemies,
        }
    }
}

/// Event fired when a fireball is cast
#[derive(Event)]
pub struct FireballCastEvent {
    pub caster: Entity,
    pub target_position: Vec3,
    pub damage: f32,
    pub radius: f32,
}

/// Marker component for fireball projectiles
#[derive(Component)]
pub struct FireballProjectile {
    pub caster: Entity,
    pub target: Vec3,
    pub damage: f32,
    pub radius: f32,
    pub speed: f32,
}

/// Tick cooldowns for fireball abilities
fn fireball_cooldown_system(
    mut query: Query<&mut AbilityCooldown, With<Fireball>>,
    time: Res<Time>,
) {
    for mut cooldown in query.iter_mut() {
        cooldown.tick(time.delta());
    }
}

/// Find targets and cast fireballs
fn fireball_targeting_system(
    mut commands: Commands,
    mut casters: Query<(
        Entity,
        &Transform,
        &Fireball,
        &mut AbilityCooldown,
        &Team,
        Option<&mut AccumulatedTags>,
    )>,
    potential_targets: Query<(Entity, &Transform, &Team)>,
) {
    for (caster_entity, caster_transform, fireball, mut cooldown, caster_team, tags) in casters.iter_mut() {
        if !cooldown.ready {
            continue;
        }

        // Find nearest enemy in range
        let mut nearest_target: Option<(Entity, Vec3, f32)> = None;

        for (target_entity, target_transform, target_team) in potential_targets.iter() {
            // Skip allies
            if caster_team == target_team {
                continue;
            }

            let distance = caster_transform.translation.distance(target_transform.translation);
            if distance <= fireball.range {
                if nearest_target.is_none() || distance < nearest_target.unwrap().2 {
                    nearest_target = Some((target_entity, target_transform.translation, distance));
                }
            }
        }

        // Cast fireball at nearest enemy
        if let Some((_, target_pos, _)) = nearest_target {
            cooldown.trigger();

            // Spawn projectile
            commands.spawn((
                FireballProjectile {
                    caster: caster_entity,
                    target: target_pos,
                    damage: fireball.damage,
                    radius: fireball.radius,
                    speed: fireball.projectile_speed,
                },
                Sprite {
                    color: Color::srgb(1.0, 0.5, 0.0), // Orange fireball
                    custom_size: Some(Vec2::splat(16.0)),
                    ..default()
                },
                Transform::from_translation(caster_transform.translation),
            ));

            // Add combat tag for research tracking
            // (tags is borrowed mutably above, so we handle it there)
        }
    }
}

/// Move fireball projectiles toward their targets
fn fireball_projectile_system(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &FireballProjectile)>,
    time: Res<Time>,
) {
    for (entity, mut transform, projectile) in projectiles.iter_mut() {
        let direction = (projectile.target - transform.translation).normalize_or_zero();
        transform.translation += direction * projectile.speed * time.delta_secs();

        // Check if reached target
        if transform.translation.distance(projectile.target) < 10.0 {
            // Mark for impact processing
            commands.entity(entity).insert(FireballImpact);
        }
    }
}

/// Marker for fireballs that have reached their target
#[derive(Component)]
struct FireballImpact;

/// Handle fireball impacts - deal AoE damage
fn fireball_impact_system(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &FireballProjectile), With<FireballImpact>>,
    targets: Query<(Entity, &Transform, &Team)>,
    casters: Query<&Team>,
    mut damage_events: EventWriter<DamageEvent>,
    mut caster_tags: Query<&mut AccumulatedTags>,
) {
    for (projectile_entity, projectile_transform, projectile) in projectiles.iter() {
        let caster_team = casters.get(projectile.caster).ok();

        // Damage all enemies in radius
        for (target_entity, target_transform, target_team) in targets.iter() {
            // Skip if same team as caster
            if let Some(ct) = caster_team {
                if ct == target_team {
                    continue;
                }
            }

            let distance = projectile_transform.translation.distance(target_transform.translation);
            if distance <= projectile.radius {
                damage_events.send(DamageEvent {
                    target: target_entity,
                    source: Some(projectile.caster),
                    amount: projectile.damage,
                    damage_type: DamageType::Fire,
                });
            }
        }

        // Track that caster dealt fire damage (for research unlocks)
        if let Ok(mut tags) = caster_tags.get_mut(projectile.caster) {
            tags.add_tag(CombatTag::DealtFireDamage);
        }

        // Despawn projectile
        commands.entity(projectile_entity).despawn();
    }
}

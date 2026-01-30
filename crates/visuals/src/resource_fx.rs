//! Resource collection visual effects - floof particles and byte generation glow

use bevy::prelude::*;
use bevy::ecs::message::MessageReader;
use ddd_core::{DamageEvent, DungeonCore, Team};
use crate::{HasVisuals, TronPalette};

/// Plugin for resource collection visual effects
pub struct ResourceFxPlugin;

impl Plugin for ResourceFxPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ByteGenerationTimer>()
            .add_systems(Update, (
                spawn_floof_particles_on_enemy_damage,
                spawn_byte_generation_particles,
                animate_floof_particles,
                animate_byte_particles,
            ));
    }
}

// ============================================================================
// Floof Collection Effect
// ============================================================================

/// Component for floof collection particle
#[derive(Component)]
pub struct FloofParticle {
    /// Timer controlling lifetime
    timer: Timer,
    /// Start position
    start_pos: Vec3,
    /// Control points for bezier curve (creates arc)
    control_point: Vec3,
}

impl FloofParticle {
    const LIFETIME: f32 = 0.5;
    const PARTICLES_PER_HIT: usize = 3;
}

/// Spawns floof particles when enemies take damage
fn spawn_floof_particles_on_enemy_damage(
    mut commands: Commands,
    mut damage_events: MessageReader<DamageEvent>,
    enemy_query: Query<(&Transform, &Team), With<HasVisuals>>,
    palette: Option<Res<TronPalette>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(palette) = palette else { return };

    for event in damage_events.read() {
        // Only spawn for enemy damage (floof source)
        if let Ok((transform, team)) = enemy_query.get(event.target) {
            if *team != Team::Enemy {
                continue;
            }

            let pos = transform.translation;

            // Scale particle count with damage (min 1, max 5)
            let particle_count = ((event.amount / 20.0) as usize).clamp(1, FloofParticle::PARTICLES_PER_HIT);

            // Floof particles are small glowing cyan orbs
            let particle_mesh = meshes.add(Sphere::new(2.0));
            let particle_material = materials.add(StandardMaterial {
                base_color: palette.cyan.with_alpha(0.9),
                emissive: TronPalette::to_emissive(palette.cyan, 5.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            // Spawn particles with random arc directions
            for i in 0..particle_count {
                // Create control point for bezier arc (rises then curves toward "UI")
                let angle = (i as f32 / particle_count as f32) * std::f32::consts::PI - std::f32::consts::FRAC_PI_2;
                let offset = Vec3::new(
                    angle.cos() * 20.0,
                    30.0 + (i as f32 * 5.0), // Rise upward
                    angle.sin() * 20.0,
                );

                commands.spawn((
                    FloofParticle {
                        timer: Timer::from_seconds(FloofParticle::LIFETIME, TimerMode::Once),
                        start_pos: pos,
                        control_point: pos + offset,
                    },
                    Mesh3d(particle_mesh.clone()),
                    MeshMaterial3d(particle_material.clone()),
                    Transform::from_translation(pos).with_scale(Vec3::splat(1.0)),
                ));
            }
        }
    }
}

/// Animates floof particles - arc upward and fade
fn animate_floof_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FloofParticle, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    // Target position (upward, toward "UI area")
    let target = Vec3::new(0.0, 100.0, -200.0);

    for (entity, mut particle, mut transform, material_handle) in query.iter_mut() {
        particle.timer.tick(time.delta());

        let t = particle.timer.fraction();

        // Quadratic bezier curve: P = (1-t)²P0 + 2(1-t)tP1 + t²P2
        let one_minus_t = 1.0 - t;
        let pos = particle.start_pos * (one_minus_t * one_minus_t)
            + particle.control_point * (2.0 * one_minus_t * t)
            + target * (t * t);

        transform.translation = pos;

        // Shrink slightly as it travels
        let scale = 1.0 - (t * 0.5);
        transform.scale = Vec3::splat(scale);

        // Increase glow as it approaches target, then fade at end
        if let Some(mat) = materials.get_mut(material_handle.id()) {
            let intensity = if t < 0.7 {
                1.0 + t
            } else {
                (1.0 - t) * 3.0
            };
            let base_emissive = 5.0 * intensity;
            mat.emissive = LinearRgba::new(
                0.4 * base_emissive,
                1.0 * base_emissive,
                1.0 * base_emissive,
                1.0,
            );

            // Fade out alpha at end
            if t > 0.7 {
                let alpha = (1.0 - t) / 0.3;
                mat.base_color = mat.base_color.with_alpha(alpha * 0.9);
            }
        }

        if particle.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Byte Generation Effect
// ============================================================================

/// Timer resource for byte particle spawning
#[derive(Resource)]
pub struct ByteGenerationTimer {
    timer: Timer,
}

impl Default for ByteGenerationTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.3, TimerMode::Repeating),
        }
    }
}

/// Component for byte generation particle
#[derive(Component)]
pub struct ByteParticle {
    /// Timer controlling lifetime
    timer: Timer,
    /// Outward velocity
    velocity: Vec3,
    /// Rotation speed
    rotation_speed: f32,
}

impl ByteParticle {
    const LIFETIME: f32 = 1.2;
    const PARTICLES_PER_BURST: usize = 4;
    const OUTWARD_SPEED: f32 = 15.0;
}

/// Spawns byte generation particles from dungeon core
fn spawn_byte_generation_particles(
    mut commands: Commands,
    core_query: Query<&Transform, (With<DungeonCore>, With<HasVisuals>)>,
    palette: Option<Res<TronPalette>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut timer: ResMut<ByteGenerationTimer>,
    time: Res<Time>,
) {
    timer.timer.tick(time.delta());

    if !timer.timer.just_finished() {
        return;
    }

    let Some(palette) = palette else { return };

    // Get core position
    let Ok(core_transform) = core_query.single() else { return };
    let core_pos = core_transform.translation;

    // Byte particles are small white-cyan cubes
    let particle_mesh = meshes.add(Cuboid::new(1.5, 1.5, 1.5));
    let particle_material = materials.add(StandardMaterial {
        base_color: palette.white_cyan.with_alpha(0.7),
        emissive: TronPalette::to_emissive(palette.white_cyan, 3.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    // Spawn particles radiating outward from core
    for i in 0..ByteParticle::PARTICLES_PER_BURST {
        let angle = (i as f32 / ByteParticle::PARTICLES_PER_BURST as f32) * std::f32::consts::TAU;
        let elevation = ((i as f32) * 1.7).sin() * 0.3; // Slight vertical variation

        let velocity = Vec3::new(
            angle.cos(),
            elevation,
            angle.sin(),
        ).normalize() * ByteParticle::OUTWARD_SPEED;

        // Start slightly outside core
        let start_offset = velocity.normalize() * 15.0;

        commands.spawn((
            ByteParticle {
                timer: Timer::from_seconds(ByteParticle::LIFETIME, TimerMode::Once),
                velocity,
                rotation_speed: 2.0 + (i as f32 * 0.5),
            },
            Mesh3d(particle_mesh.clone()),
            MeshMaterial3d(particle_material.clone()),
            Transform::from_translation(core_pos + start_offset).with_scale(Vec3::splat(1.0)),
        ));
    }
}

/// Animates byte particles - radiate outward, spin, and fade
fn animate_byte_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ByteParticle, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut transform, material_handle) in query.iter_mut() {
        particle.timer.tick(time.delta());

        let t = particle.timer.fraction();

        // Move outward (slowing down over time)
        let speed_falloff = 1.0 - (t * 0.7);
        transform.translation += particle.velocity * dt * speed_falloff;

        // Spin
        transform.rotate_local_y(dt * particle.rotation_speed);
        transform.rotate_local_x(dt * particle.rotation_speed * 0.7);

        // Scale: start small, grow, then shrink
        let scale = if t < 0.2 {
            t / 0.2 // Grow in
        } else if t < 0.7 {
            1.0 // Full size
        } else {
            (1.0 - t) / 0.3 // Shrink out
        };
        transform.scale = Vec3::splat(scale.max(0.1));

        // Fade out
        if let Some(mat) = materials.get_mut(material_handle.id()) {
            let alpha = if t < 0.7 { 0.7 } else { (1.0 - t) / 0.3 * 0.7 };
            mat.base_color = mat.base_color.with_alpha(alpha);

            // Reduce emissive as particle fades
            let emissive_mult = if t < 0.5 { 1.0 } else { (1.0 - t) * 2.0 };
            mat.emissive = LinearRgba::new(
                0.8 * 3.0 * emissive_mult,
                1.0 * 3.0 * emissive_mult,
                1.0 * 3.0 * emissive_mult,
                1.0,
            );
        }

        if particle.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

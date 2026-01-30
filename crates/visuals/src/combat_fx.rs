//! Combat visual effects - hit flashes, damage numbers, death particles

use bevy::prelude::*;
use bevy::ecs::message::MessageReader;
use ddd_core::{DamageEvent, DeathEvent, Team};
use crate::{HasVisuals, TronPalette};

/// Plugin for combat visual effects
pub struct CombatFxPlugin;

impl Plugin for CombatFxPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                spawn_hit_flash_on_damage,
                spawn_damage_number_on_damage,
                spawn_death_particles_on_death,
                animate_hit_flash,
                animate_damage_numbers,
                animate_death_particles,
            ));
    }
}

// ============================================================================
// Hit Flash Effect
// ============================================================================

/// Component for hit flash animation state
#[derive(Component)]
pub struct HitFlash {
    /// Timer controlling flash duration
    timer: Timer,
    /// Original base color to restore after flash
    original_color: Color,
    /// Original emissive to restore
    original_emissive: LinearRgba,
}

impl HitFlash {
    const FLASH_DURATION: f32 = 0.12;
}

/// Spawns hit flash effect when entities take damage
fn spawn_hit_flash_on_damage(
    mut commands: Commands,
    mut damage_events: MessageReader<DamageEvent>,
    query: Query<(Entity, &MeshMaterial3d<StandardMaterial>), With<HasVisuals>>,
    materials: Res<Assets<StandardMaterial>>,
) {
    for event in damage_events.read() {
        // Find the visual entity that was damaged
        if let Ok((entity, material_handle)) = query.get(event.target) {
            // Get original material colors
            if let Some(mat) = materials.get(material_handle.id()) {
                commands.entity(entity).insert(HitFlash {
                    timer: Timer::from_seconds(HitFlash::FLASH_DURATION, TimerMode::Once),
                    original_color: mat.base_color,
                    original_emissive: mat.emissive,
                });
            }
        }
    }
}

/// Animates hit flash - white flash that fades back to original
fn animate_hit_flash(
    mut commands: Commands,
    mut query: Query<(Entity, &mut HitFlash, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    const FLASH_COLOR: Color = Color::WHITE;
    const FLASH_EMISSIVE_INTENSITY: f32 = 8.0;

    for (entity, mut flash, material_handle) in query.iter_mut() {
        flash.timer.tick(time.delta());

        if let Some(mat) = materials.get_mut(material_handle.id()) {
            if flash.timer.is_finished() {
                // Restore original colors
                mat.base_color = flash.original_color;
                mat.emissive = flash.original_emissive;
                commands.entity(entity).remove::<HitFlash>();
            } else {
                // Lerp from flash color back to original
                let t = flash.timer.fraction();
                mat.base_color = lerp_color(FLASH_COLOR, flash.original_color, t);

                let flash_emissive = LinearRgba::new(
                    FLASH_EMISSIVE_INTENSITY,
                    FLASH_EMISSIVE_INTENSITY,
                    FLASH_EMISSIVE_INTENSITY,
                    1.0,
                );
                mat.emissive = lerp_linear_rgba(flash_emissive, flash.original_emissive, t);
            }
        }
    }
}

// ============================================================================
// Damage Numbers
// ============================================================================

/// Component for floating damage number
#[derive(Component)]
pub struct DamageNumber {
    /// Timer controlling lifetime
    timer: Timer,
    /// Starting Y position
    start_y: f32,
}

impl DamageNumber {
    const LIFETIME: f32 = 0.8;
    const RISE_HEIGHT: f32 = 30.0;
}

/// Marker for damage number text
#[derive(Component)]
pub struct DamageNumberText;

/// Spawns floating damage numbers when entities take damage
fn spawn_damage_number_on_damage(
    mut commands: Commands,
    mut damage_events: MessageReader<DamageEvent>,
    query: Query<&Transform, With<HasVisuals>>,
    palette: Option<Res<TronPalette>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(palette) = palette else { return };

    for event in damage_events.read() {
        // Get position of damaged entity
        if let Ok(transform) = query.get(event.target) {
            let pos = transform.translation;

            // Create a simple glowing sphere as damage indicator
            // (Text rendering in 3D would require additional setup)
            let size = (event.amount / 10.0).clamp(1.0, 5.0);
            let mesh = meshes.add(Sphere::new(size));

            // Color based on damage amount: more damage = more intense
            let intensity = (event.amount / 50.0).clamp(0.5, 2.0);
            let material = materials.add(StandardMaterial {
                base_color: palette.orange.with_alpha(0.9),
                emissive: TronPalette::to_emissive(palette.orange, 3.0 * intensity),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            commands.spawn((
                DamageNumber {
                    timer: Timer::from_seconds(DamageNumber::LIFETIME, TimerMode::Once),
                    start_y: pos.y + 10.0,
                },
                DamageNumberText,
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(Vec3::new(pos.x, pos.y + 10.0, pos.z)),
            ));
        }
    }
}

/// Animates damage numbers - rise and fade out
fn animate_damage_numbers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DamageNumber, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut damage_num, mut transform, material_handle) in query.iter_mut() {
        damage_num.timer.tick(time.delta());

        let t = damage_num.timer.fraction();

        // Rise up
        transform.translation.y = damage_num.start_y + t * DamageNumber::RISE_HEIGHT;

        // Scale down as it rises
        let scale = 1.0 - (t * 0.5);
        transform.scale = Vec3::splat(scale);

        // Fade out
        if let Some(mat) = materials.get_mut(material_handle.id()) {
            let alpha = 1.0 - t;
            mat.base_color = mat.base_color.with_alpha(alpha * 0.9);
            mat.emissive = LinearRgba::new(
                mat.emissive.red * alpha,
                mat.emissive.green * alpha,
                mat.emissive.blue * alpha,
                1.0,
            );
        }

        if damage_num.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Death Particles
// ============================================================================

/// Component for death particle burst
#[derive(Component)]
pub struct DeathParticle {
    /// Timer controlling lifetime
    timer: Timer,
    /// Velocity of this particle
    velocity: Vec3,
}

impl DeathParticle {
    const LIFETIME: f32 = 0.6;
    const PARTICLE_COUNT: usize = 12;
    const BURST_SPEED: f32 = 80.0;
}

/// Spawns death particle burst when entities die
fn spawn_death_particles_on_death(
    mut commands: Commands,
    mut death_events: MessageReader<DeathEvent>,
    query: Query<(&Transform, Option<&Team>), With<HasVisuals>>,
    palette: Option<Res<TronPalette>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(palette) = palette else { return };

    for event in death_events.read() {
        // Get position and team of dead entity
        if let Ok((transform, team)) = query.get(event.entity) {
            let pos = transform.translation;

            // Color based on team
            let color = match team {
                Some(Team::Player) => palette.cyan,
                Some(Team::Enemy) => palette.orange,
                None => palette.magenta,
            };

            // Create particle mesh (small tetrahedron for interesting shape)
            let particle_mesh = meshes.add(Tetrahedron::default());

            let particle_material = materials.add(StandardMaterial {
                base_color: color.with_alpha(0.8),
                emissive: TronPalette::to_emissive(color, 4.0),
                alpha_mode: AlphaMode::Blend,
                ..default()
            });

            // Spawn burst of particles in random directions
            for i in 0..DeathParticle::PARTICLE_COUNT {
                // Distribute particles in a sphere
                let angle = (i as f32 / DeathParticle::PARTICLE_COUNT as f32) * std::f32::consts::TAU;
                let elevation = (i as f32 * 0.3).sin() * 0.5;

                let velocity = Vec3::new(
                    angle.cos() * (1.0 - elevation.abs()),
                    elevation + 0.3, // Bias upward slightly
                    angle.sin() * (1.0 - elevation.abs()),
                ).normalize() * DeathParticle::BURST_SPEED;

                commands.spawn((
                    DeathParticle {
                        timer: Timer::from_seconds(DeathParticle::LIFETIME, TimerMode::Once),
                        velocity,
                    },
                    Mesh3d(particle_mesh.clone()),
                    MeshMaterial3d(particle_material.clone()),
                    Transform::from_translation(pos).with_scale(Vec3::splat(3.0)),
                ));
            }
        }
    }
}

/// Animates death particles - fly outward and fade
fn animate_death_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DeathParticle, &mut Transform, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut transform, material_handle) in query.iter_mut() {
        particle.timer.tick(time.delta());

        let t = particle.timer.fraction();

        // Move outward with gravity
        let gravity = Vec3::new(0.0, -50.0, 0.0);
        transform.translation += particle.velocity * dt;
        particle.velocity += gravity * dt;

        // Shrink and spin
        let scale = (1.0 - t) * 3.0;
        transform.scale = Vec3::splat(scale.max(0.1));
        transform.rotate_local_y(dt * 5.0);
        transform.rotate_local_x(dt * 3.0);

        // Fade out
        if let Some(mat) = materials.get_mut(material_handle.id()) {
            let alpha = 1.0 - t;
            mat.base_color = mat.base_color.with_alpha(alpha * 0.8);
            mat.emissive = LinearRgba::new(
                mat.emissive.red * (1.0 - t * 0.5),
                mat.emissive.green * (1.0 - t * 0.5),
                mat.emissive.blue * (1.0 - t * 0.5),
                1.0,
            );
        }

        if particle.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Lerp between two colors
fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    )
}

/// Lerp between two LinearRgba values
fn lerp_linear_rgba(a: LinearRgba, b: LinearRgba, t: f32) -> LinearRgba {
    LinearRgba::new(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    )
}

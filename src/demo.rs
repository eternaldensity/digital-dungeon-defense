//! Demo scene for testing the game systems

use bevy::prelude::*;
use bevy::ecs::message::MessageReader;
use ddd_core::{
    DeathEvent, PlayerResources, SpawnWaveEnemyEvent, WavePhase, WaveState,
};
use ddd_creatures::{BasicFloofSpawnerBundle, Virus, VirusBundle};
use ddd_traps::{DataSpikeBundle, FirewallBundle};

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_demo_scene)
            .add_systems(Update, (
                handle_wave_enemy_spawns,
                track_enemy_deaths,
                display_resources,
            ));
    }
}

/// Sets up a demo scene with spawners, traps, and enemies
fn setup_demo_scene(mut commands: Commands) {
    // Spawn some FloofByte spawners
    commands.spawn(BasicFloofSpawnerBundle::new(
        Vec3::new(-100.0, 50.0, 0.0),
        0,
    ));

    commands.spawn(BasicFloofSpawnerBundle::new(
        Vec3::new(-100.0, -50.0, 0.0),
        0,
    ));

    // Spawn some traps
    commands.spawn(FirewallBundle::new(
        Vec3::new(50.0, 0.0, 0.0),
        0,
    ));

    commands.spawn(DataSpikeBundle::new(
        Vec3::new(100.0, 50.0, 0.0),
        0,
    ));

    commands.spawn(DataSpikeBundle::new(
        Vec3::new(100.0, -50.0, 0.0),
        0,
    ));

    // Add UI text for resources and wave info
    commands.spawn((
        Text::new("Resources: "),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ResourceText,
    ));

    // Add wave status text
    commands.spawn((
        Text::new("Wave: "),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.8, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(35.0),
            left: Val::Px(10.0),
            ..default()
        },
        WaveText,
    ));

    // Add instructions
    commands.spawn((
        Text::new("Digital Dungeon Defense - Demo\nGreen: FloofBytes (defenders)\nRed: Viruses (enemies)\nBlue: Dungeon Core\nOrange: Firewall trap\nPurple: Data Spike trap"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

/// Marker for resource display text
#[derive(Component)]
struct ResourceText;

/// Marker for wave status text
#[derive(Component)]
struct WaveText;

/// Handles spawning enemies from wave events
fn handle_wave_enemy_spawns(
    mut commands: Commands,
    mut spawn_events: MessageReader<SpawnWaveEnemyEvent>,
) {
    for event in spawn_events.read() {
        commands.spawn(VirusBundle::new(event.position, event.floor));
    }
}

/// Tracks enemy deaths to update wave state
fn track_enemy_deaths(
    mut death_events: MessageReader<DeathEvent>,
    mut wave_state: ResMut<WaveState>,
    enemies: Query<&Virus>,
) {
    for event in death_events.read() {
        // Check if the dead entity was an enemy
        if enemies.get(event.entity).is_ok() {
            wave_state.enemy_died();
        }
    }
}

/// Updates the resource and wave display
fn display_resources(
    resources: Res<PlayerResources>,
    wave_state: Res<WaveState>,
    mut resource_query: Query<&mut Text, (With<ResourceText>, Without<WaveText>)>,
    mut wave_query: Query<&mut Text, (With<WaveText>, Without<ResourceText>)>,
) {
    for mut text in resource_query.iter_mut() {
        **text = format!(
            "Bytes: {:.0}/{:.0} | Floof: {:.0} | Chips: {} | Core Lvl: {}",
            resources.bytes,
            resources.byte_capacity,
            resources.floof,
            resources.chips,
            resources.core_level,
        );
    }

    for mut text in wave_query.iter_mut() {
        let phase_str = match wave_state.phase {
            WavePhase::Preparing => "Preparing...",
            WavePhase::Active => "Active",
            WavePhase::Complete => "Complete!",
            WavePhase::GameOver => "GAME OVER",
            WavePhase::Victory => "VICTORY!",
        };

        **text = format!(
            "Wave {} | {} | Enemies: {}",
            wave_state.current_wave + 1,
            phase_str,
            wave_state.enemies_alive,
        );
    }
}

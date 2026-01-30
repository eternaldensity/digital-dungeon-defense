//! Demo scene for testing the game systems

use bevy::prelude::*;
use ddd_core::{DungeonGrid, PlayerResources};
use ddd_creatures::{BasicFloofSpawnerBundle, VirusBundle};
use ddd_traps::{DataSpikeBundle, FirewallBundle};

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_demo_scene)
            .add_systems(Update, (
                spawn_enemy_waves,
                display_resources,
            ));
    }
}

/// Wave spawner state
#[derive(Resource)]
struct WaveState {
    timer: Timer,
    wave_number: u32,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(10.0, TimerMode::Repeating),
            wave_number: 0,
        }
    }
}

/// Sets up a demo scene with spawners, traps, and enemies
fn setup_demo_scene(
    mut commands: Commands,
    dungeon: Res<DungeonGrid>,
) {
    // Initialize wave state
    commands.insert_resource(WaveState::default());

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

    // Spawn initial enemies at the edge
    let spawn_positions = vec![
        Vec3::new(300.0, 100.0, 0.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(300.0, -100.0, 0.0),
    ];

    for pos in spawn_positions {
        commands.spawn(VirusBundle::new(pos, 0));
    }

    // Add UI text for resources
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

/// Spawns waves of enemies periodically
fn spawn_enemy_waves(
    mut commands: Commands,
    mut wave_state: ResMut<WaveState>,
    time: Res<Time>,
) {
    wave_state.timer.tick(time.delta());

    if wave_state.timer.just_finished() {
        wave_state.wave_number += 1;

        // Spawn more enemies each wave
        let enemy_count = 2 + wave_state.wave_number;

        for i in 0..enemy_count {
            let y_offset = (i as f32 - enemy_count as f32 / 2.0) * 40.0;
            commands.spawn(VirusBundle::new(
                Vec3::new(350.0, y_offset, 0.0),
                0,
            ));
        }
    }
}

/// Updates the resource display
fn display_resources(
    resources: Res<PlayerResources>,
    mut query: Query<&mut Text, With<ResourceText>>,
) {
    for mut text in query.iter_mut() {
        **text = format!(
            "Bytes: {:.0}/{:.0} | Floof: {:.0} | Chips: {} | Core Lvl: {}",
            resources.bytes,
            resources.byte_capacity,
            resources.floof,
            resources.chips,
            resources.core_level,
        );
    }
}

//! Digital Dungeon Defense - Main entry point
//!
//! A tower defense game where you build a dungeon to defend against enemy invasions.

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use ddd_abilities::AbilitiesPlugin;
use ddd_building::BuildingPlugin;
use ddd_core::{
    DeathEvent, GameOverEvent, GameOverState, GameState, PlayerResources, SpawnWaveEnemyEvent,
    WavePhase, WaveState,
};
use ddd_creatures::{CreaturesPlugin, Virus, VirusBundle};
use ddd_floors::FloorsPlugin;
use ddd_traps::TrapsPlugin;
use ddd_visuals::VisualsPlugin;

mod layout_compiler;
use layout_compiler::{compile_layout, spawn_gameplay_entities};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Digital Dungeon Defense".into(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
        // Game plugins - each self-contained and extensible
        .add_plugins((
            CorePlugin,
            BuildingPlugin,
            CreaturesPlugin,
            AbilitiesPlugin,
            FloorsPlugin,
            TrapsPlugin,
            VisualsPlugin,
        ))
        // Game flow plugin
        .add_plugins(GameFlowPlugin)
        .run();
}

use ddd_core::CorePlugin;

/// Plugin that manages the game flow: title screen, building->active transition, game over.
pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app
            // Title screen UI
            .add_systems(OnEnter(GameState::Title), setup_title_screen)
            .add_systems(OnExit(GameState::Title), cleanup_title_screen)
            // Building phase UI
            .add_systems(OnEnter(GameState::Building), setup_building_ui)
            .add_systems(OnExit(GameState::Building), cleanup_building_ui)
            // Building -> Active transition (when build mode exits with valid layout)
            .add_systems(
                Update,
                handle_build_mode_exit.run_if(in_state(GameState::Building)),
            )
            // Active phase - compile layout and spawn entities
            .add_systems(OnEnter(GameState::Active), on_enter_active)
            .add_systems(OnExit(GameState::Active), cleanup_active_phase)
            // Game over screen
            .add_systems(OnEnter(GameState::GameOver), setup_game_over_screen)
            .add_systems(OnExit(GameState::GameOver), cleanup_game_over_screen)
            // Active gameplay systems
            .add_systems(
                Update,
                (
                    handle_wave_enemy_spawns,
                    track_enemy_deaths,
                    display_game_info,
                    check_wave_game_over,
                )
                    .run_if(in_state(GameState::Active)),
            );
    }
}

// =============================================================================
// Title Screen
// =============================================================================

#[derive(Component)]
struct TitleScreenUI;

fn setup_title_screen(mut commands: Commands) {
    // Root container - full screen centered flex column
    commands
        .spawn((
            TitleScreenUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.02, 0.95)),
        ))
        .with_children(|parent| {
            // Title text
            parent.spawn((
                Text::new("DIGITAL DUNGEON DEFENSE"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.8, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("Build your dungeon to defend against invaders!"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Start prompt (highlighted)
            parent.spawn((
                Text::new("[ Press SPACE to Start ]"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // Controls section
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.0)),
                    ..default()
                },))
                .with_children(|controls| {
                    controls.spawn((
                        Text::new("Controls"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.8, 0.9)),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    let control_lines = [
                        "T: Open tile menu",
                        "Click: Place tile  |  Right-click: Remove",
                        "R: Rotate  |  F: Flip",
                        "SPACE: Start waves (when layout is valid)",
                    ];

                    for line in control_lines {
                        controls.spawn((
                            Text::new(line),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.4, 0.4, 0.4)),
                            Node {
                                margin: UiRect::bottom(Val::Px(6.0)),
                                ..default()
                            },
                        ));
                    }
                });
        });
}

fn cleanup_title_screen(mut commands: Commands, query: Query<Entity, With<TitleScreenUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_related::<Children>();
        commands.entity(entity).despawn();
    }
}

// =============================================================================
// Building Phase UI
// =============================================================================

#[derive(Component)]
struct BuildingUI;

fn setup_building_ui(
    mut commands: Commands,
    mut build_mode: ResMut<NextState<ddd_building::BuildMode>>,
) {
    // Enable build mode
    build_mode.set(ddd_building::BuildMode::View);

    // Building mode instructions
    commands.spawn((
        Text::new("BUILDING MODE\n\nT: Open tile menu\nClick: Place tile\nRight-click: Remove tile\nR: Rotate  F: Flip\nSPACE: Start waves (when valid)"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.2)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
        BuildingUI,
    ));

}

fn cleanup_building_ui(mut commands: Commands, query: Query<Entity, With<BuildingUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Watch for build mode exiting (Space/Enter with valid layout) and transition to Active.
fn handle_build_mode_exit(
    mut build_mode_events: MessageReader<ddd_building::BuildModeChangedEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in build_mode_events.read() {
        // When build mode is disabled (player pressed Space with valid layout), start active phase
        if !event.enabled {
            next_state.set(GameState::Active);
        }
    }
}

// =============================================================================
// Active Phase
// =============================================================================

#[derive(Component)]
struct ActivePhaseEntity;

#[derive(Component)]
struct ResourceText;

#[derive(Component)]
struct WaveText;

fn on_enter_active(
    mut commands: Commands,
    layout: Res<ddd_building::DungeonLayout>,
    mut grid: ResMut<ddd_core::DungeonGrid>,
    mut wave_state: ResMut<WaveState>,
) {
    // Compile the planning layout into the gameplay grid
    let result = compile_layout(&layout, &mut grid);

    // Spawn gameplay entities (traps, spawners, core) from the layout
    let _spawned = spawn_gameplay_entities(&mut commands, &layout, &grid);

    // Set up wave state with spawn points from up connectors
    wave_state.enemy_spawn_points = result.enemy_spawn_points;
    wave_state.current_wave = 0;
    wave_state.phase = WavePhase::Preparing;
    wave_state.prep_timer.reset();
    wave_state.enemies_spawned = 0;
    wave_state.enemies_alive = 0;

    // Spawn resource display UI
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
        ActivePhaseEntity,
    ));

    // Spawn wave status UI
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
        ActivePhaseEntity,
    ));
}

fn cleanup_active_phase(
    mut commands: Commands,
    query: Query<Entity, With<ActivePhaseEntity>>,
    gameplay_entities: Query<
        Entity,
        Or<(
            With<ddd_core::DungeonCore>,
            With<ddd_traps::Trap>,
            With<ddd_creatures::Spawner>,
            With<Virus>,
            With<ddd_creatures::BasicFloof>,
        )>,
    >,
) {
    // Clean up UI
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Clean up all gameplay entities
    for entity in gameplay_entities.iter() {
        commands.entity(entity).despawn();
    }
}

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
        if enemies.get(event.entity).is_ok() {
            wave_state.enemy_died();
        }
    }
}

/// Updates the resource and wave display
fn display_game_info(
    resources: Res<PlayerResources>,
    wave_state: Res<WaveState>,
    mut resource_query: Query<&mut Text, (With<ResourceText>, Without<WaveText>)>,
    mut wave_query: Query<&mut Text, (With<WaveText>, Without<ResourceText>)>,
) {
    for mut text in resource_query.iter_mut() {
        **text = format!(
            "Bytes: {:.0}/{:.0} | Floof: {:.0} | Chips: {}",
            resources.bytes, resources.byte_capacity, resources.floof, resources.chips,
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

/// Check for game over conditions and transition state
fn check_wave_game_over(
    mut game_over_events: MessageReader<GameOverEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_over_state: ResMut<GameOverState>,
) {
    for event in game_over_events.read() {
        *game_over_state = if event.victory {
            GameOverState::victory(event.final_wave)
        } else {
            GameOverState::defeat(event.final_wave)
        };
        next_state.set(GameState::GameOver);
    }
}

// =============================================================================
// Game Over Screen
// =============================================================================

#[derive(Component)]
struct GameOverUI;

fn setup_game_over_screen(mut commands: Commands, game_over_state: Res<GameOverState>) {
    let (title, color) = if game_over_state.victory {
        ("VICTORY!", Color::srgb(0.2, 1.0, 0.2))
    } else {
        ("DEFEAT!", Color::srgb(1.0, 0.2, 0.2))
    };

    // Root container - full screen centered flex column
    commands
        .spawn((
            GameOverUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.02, 0.95)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(color),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Wave reached
            parent.spawn((
                Text::new(format!("Wave {} reached", game_over_state.final_wave + 1)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Restart prompt
            parent.spawn((
                Text::new("[ Press SPACE to play again ]"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn cleanup_game_over_screen(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_related::<Children>();
        commands.entity(entity).despawn();
    }
}

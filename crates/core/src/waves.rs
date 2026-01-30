//! Wave spawning system for enemy progression

use bevy::prelude::*;
use bevy::ecs::message::{Message, MessageReader, MessageWriter};

use crate::{DeathEvent, DungeonCore};

/// Current state of the wave system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WavePhase {
    /// Time before wave starts (preparation phase)
    #[default]
    Preparing,
    /// Wave is actively spawning/fighting
    Active,
    /// All enemies defeated, waiting for next wave
    Complete,
    /// Game over - core destroyed
    GameOver,
    /// All waves completed - victory
    Victory,
}

/// Configuration for a single wave
#[derive(Clone, Debug)]
pub struct WaveConfig {
    /// Number of enemies to spawn this wave
    pub enemy_count: u32,
    /// Delay between enemy spawns in seconds
    pub spawn_delay: f32,
    /// Base health multiplier for enemies
    pub health_multiplier: f32,
    /// Base damage multiplier for enemies
    pub damage_multiplier: f32,
}

impl WaveConfig {
    pub fn for_wave(wave_number: u32) -> Self {
        // Scale difficulty with wave number
        let base_enemies = 3;
        let enemies_per_wave = 2;

        Self {
            enemy_count: base_enemies + wave_number * enemies_per_wave,
            spawn_delay: (1.5 - wave_number as f32 * 0.1).max(0.3),
            health_multiplier: 1.0 + wave_number as f32 * 0.15,
            damage_multiplier: 1.0 + wave_number as f32 * 0.1,
        }
    }
}

/// Resource tracking wave progression
#[derive(Resource)]
pub struct WaveState {
    /// Current wave number (0-indexed)
    pub current_wave: u32,
    /// Maximum waves (None for endless mode)
    pub max_waves: Option<u32>,
    /// Current phase of the wave
    pub phase: WavePhase,
    /// Timer for preparation phase
    pub prep_timer: Timer,
    /// Timer for spawn delay between enemies
    pub spawn_timer: Timer,
    /// Number of enemies spawned this wave
    pub enemies_spawned: u32,
    /// Number of enemies remaining alive
    pub enemies_alive: u32,
    /// Current wave configuration
    pub config: WaveConfig,
    /// Enemy spawn points from compiled layout (world position, floor index).
    /// If empty, enemies spawn at default edge positions.
    pub enemy_spawn_points: Vec<(Vec3, usize)>,
}

impl Default for WaveState {
    fn default() -> Self {
        Self {
            current_wave: 0,
            max_waves: None, // Endless mode by default
            phase: WavePhase::Preparing,
            prep_timer: Timer::from_seconds(5.0, TimerMode::Once),
            spawn_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            enemies_spawned: 0,
            enemies_alive: 0,
            config: WaveConfig::for_wave(0),
            enemy_spawn_points: Vec::new(),
        }
    }
}

impl WaveState {
    pub fn new(max_waves: Option<u32>) -> Self {
        Self {
            max_waves,
            ..default()
        }
    }

    /// Start the next wave
    pub fn start_wave(&mut self) {
        self.config = WaveConfig::for_wave(self.current_wave);
        self.spawn_timer = Timer::from_seconds(self.config.spawn_delay, TimerMode::Repeating);
        self.enemies_spawned = 0;
        self.phase = WavePhase::Active;
    }

    /// Called when an enemy is spawned
    pub fn enemy_spawned(&mut self) {
        self.enemies_spawned += 1;
        self.enemies_alive += 1;
    }

    /// Called when an enemy dies
    pub fn enemy_died(&mut self) {
        self.enemies_alive = self.enemies_alive.saturating_sub(1);
    }

    /// Check if wave is complete
    pub fn is_wave_complete(&self) -> bool {
        self.enemies_spawned >= self.config.enemy_count && self.enemies_alive == 0
    }

    /// Check if all spawning is done for this wave
    pub fn is_spawning_complete(&self) -> bool {
        self.enemies_spawned >= self.config.enemy_count
    }

    /// Advance to the next wave
    pub fn advance_wave(&mut self) {
        self.current_wave += 1;
        self.prep_timer.reset();
        self.phase = WavePhase::Preparing;

        // Check for victory
        if let Some(max) = self.max_waves {
            if self.current_wave >= max {
                self.phase = WavePhase::Victory;
            }
        }
    }

    /// Set game over state
    pub fn game_over(&mut self) {
        self.phase = WavePhase::GameOver;
    }
}

/// Message fired when a wave starts
#[derive(Message)]
pub struct WaveStartEvent {
    pub wave_number: u32,
    pub config: WaveConfig,
}

/// Message fired when a wave is completed
#[derive(Message)]
pub struct WaveCompleteEvent {
    pub wave_number: u32,
}

/// Message requesting an enemy spawn at a position
#[derive(Message)]
pub struct SpawnWaveEnemyEvent {
    pub position: Vec3,
    pub floor: usize,
    pub health_multiplier: f32,
    pub damage_multiplier: f32,
}

/// Message fired when game is over
#[derive(Message)]
pub struct GameOverEvent {
    pub final_wave: u32,
    pub victory: bool,
}

/// System that manages wave state transitions
pub fn wave_state_system(
    mut wave_state: ResMut<WaveState>,
    mut wave_start_events: MessageWriter<WaveStartEvent>,
    mut wave_complete_events: MessageWriter<WaveCompleteEvent>,
    mut game_over_events: MessageWriter<GameOverEvent>,
    time: Res<Time>,
) {
    match wave_state.phase {
        WavePhase::Preparing => {
            wave_state.prep_timer.tick(time.delta());
            if wave_state.prep_timer.is_finished() {
                wave_state.start_wave();
                wave_start_events.write(WaveStartEvent {
                    wave_number: wave_state.current_wave,
                    config: wave_state.config.clone(),
                });
            }
        }
        WavePhase::Active => {
            if wave_state.is_wave_complete() {
                wave_complete_events.write(WaveCompleteEvent {
                    wave_number: wave_state.current_wave,
                });
                wave_state.phase = WavePhase::Complete;
            }
        }
        WavePhase::Complete => {
            // Short delay before next wave
            wave_state.prep_timer.reset();
            wave_state.advance_wave();

            // Check for game end states
            if wave_state.phase == WavePhase::Victory {
                game_over_events.write(GameOverEvent {
                    final_wave: wave_state.current_wave,
                    victory: true,
                });
            }
        }
        WavePhase::GameOver | WavePhase::Victory => {
            // No state transitions in end states
        }
    }
}

/// System that spawns enemies during active waves
pub fn wave_spawn_system(
    mut wave_state: ResMut<WaveState>,
    mut spawn_events: MessageWriter<SpawnWaveEnemyEvent>,
    time: Res<Time>,
) {
    if wave_state.phase != WavePhase::Active {
        return;
    }

    if wave_state.is_spawning_complete() {
        return;
    }

    wave_state.spawn_timer.tick(time.delta());

    if wave_state.spawn_timer.just_finished() {
        // Get spawn position from enemy_spawn_points (up connector on top floor)
        // or fall back to default edge position
        let (position, floor) = if wave_state.enemy_spawn_points.is_empty() {
            // Fallback: spawn at edge of screen
            let spawn_index = wave_state.enemies_spawned;
            let y_offset =
                (spawn_index as f32 - wave_state.config.enemy_count as f32 / 2.0) * 40.0;
            (Vec3::new(350.0, y_offset, 0.0), 0)
        } else {
            // Cycle through configured spawn points (up connectors on top floor)
            let spawn_idx =
                wave_state.enemies_spawned as usize % wave_state.enemy_spawn_points.len();
            wave_state.enemy_spawn_points[spawn_idx]
        };

        spawn_events.write(SpawnWaveEnemyEvent {
            position,
            floor,
            health_multiplier: wave_state.config.health_multiplier,
            damage_multiplier: wave_state.config.damage_multiplier,
        });

        wave_state.enemy_spawned();
    }
}

/// System that detects when the dungeon core is destroyed
pub fn core_death_system(
    mut death_events: MessageReader<DeathEvent>,
    mut wave_state: ResMut<WaveState>,
    mut game_over_events: MessageWriter<GameOverEvent>,
    cores: Query<Entity, With<DungeonCore>>,
) {
    for event in death_events.read() {
        if cores.get(event.entity).is_ok() {
            wave_state.game_over();
            game_over_events.write(GameOverEvent {
                final_wave: wave_state.current_wave,
                victory: false,
            });
        }
    }
}

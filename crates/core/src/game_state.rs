//! Game state management for the main gameplay loop.
//!
//! Controls the flow: Title → Building → Active → GameOver

use bevy::prelude::*;

/// Main game state controlling the overall flow.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, States, Hash)]
pub enum GameState {
    /// Title screen - press Space to start
    #[default]
    Title,
    /// Building phase - place tiles, traps, spawners
    Building,
    /// Active gameplay - waves attacking
    Active,
    /// Game paused (overlay during Active)
    Paused,
    /// Game ended (victory or defeat)
    GameOver,
}

/// Tracks whether the game ended in victory or defeat.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Resource)]
pub struct GameOverState {
    pub victory: bool,
    pub final_wave: u32,
}

impl GameOverState {
    pub fn victory(wave: u32) -> Self {
        Self {
            victory: true,
            final_wave: wave,
        }
    }

    pub fn defeat(wave: u32) -> Self {
        Self {
            victory: false,
            final_wave: wave,
        }
    }
}

/// System that handles title screen input.
pub fn title_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Building);
    }
}

/// System that handles game over screen input.
pub fn game_over_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Escape)
    {
        next_state.set(GameState::Title);
    }
}

/// System that handles pause toggle during active gameplay.
pub fn pause_toggle_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        match *current_state.get() {
            GameState::Active => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Active),
            _ => {}
        }
    }
}

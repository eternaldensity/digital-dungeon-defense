//! Base trap systems and components shared by all traps

use bevy::prelude::*;
use bevy::ecs::message::{Message, MessageReader};
use ddd_core::PlayerResources;

pub struct TrapBasePlugin;

impl Plugin for TrapBasePlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<Trap>()
            .add_message::<TrapTriggeredEvent>()
            .add_message::<TrapResetEvent>()
            .add_systems(Update, (
                trap_cooldown_system,
                trap_reset_system,
            ));
    }
}

/// Base component for all traps
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Trap {
    /// Whether the trap is currently active and can trigger
    pub active: bool,
    /// Whether the trap is on cooldown after triggering
    pub on_cooldown: bool,
    /// Cooldown timer after triggering
    pub cooldown: Timer,
    /// Cost in bytes to reset/repair
    pub reset_cost: f32,
    /// Cost in floof to construct
    pub build_cost: f32,
}

impl Default for Trap {
    fn default() -> Self {
        Self {
            active: true,
            on_cooldown: false,
            cooldown: Timer::from_seconds(3.0, TimerMode::Once),
            reset_cost: 10.0,
            build_cost: 25.0,
        }
    }
}

impl Trap {
    pub fn new(cooldown_secs: f32, reset_cost: f32, build_cost: f32) -> Self {
        Self {
            active: true,
            on_cooldown: false,
            cooldown: Timer::from_seconds(cooldown_secs, TimerMode::Once),
            reset_cost,
            build_cost,
        }
    }

    pub fn can_trigger(&self) -> bool {
        self.active && !self.on_cooldown
    }

    pub fn trigger(&mut self) {
        self.on_cooldown = true;
        self.cooldown.reset();
    }
}

/// Message fired when a trap is triggered
#[derive(Message)]
pub struct TrapTriggeredEvent {
    pub trap: Entity,
    pub triggered_by: Entity,
}

/// Message to request resetting a trap (costs bytes)
#[derive(Message)]
pub struct TrapResetEvent {
    pub trap: Entity,
}

/// Marker for traps that need manual reset (don't auto-rearm)
#[derive(Component)]
pub struct ManualReset;

/// Marker for traps that can be destroyed
#[derive(Component)]
pub struct Destructible;

/// System that handles trap cooldowns
fn trap_cooldown_system(
    mut traps: Query<&mut Trap, Without<ManualReset>>,
    time: Res<Time>,
) {
    for mut trap in traps.iter_mut() {
        if trap.on_cooldown {
            trap.cooldown.tick(time.delta());
            if trap.cooldown.is_finished() {
                trap.on_cooldown = false;
            }
        }
    }
}

/// System that handles manual trap resets
fn trap_reset_system(
    mut reset_events: MessageReader<TrapResetEvent>,
    mut traps: Query<&mut Trap>,
    mut resources: ResMut<PlayerResources>,
) {
    for event in reset_events.read() {
        if let Ok(mut trap) = traps.get_mut(event.trap) {
            if resources.try_spend_bytes(trap.reset_cost) {
                trap.active = true;
                trap.on_cooldown = false;
                trap.cooldown.reset();
            }
        }
    }
}

/// Trigger zone component - detects when enemies enter
#[derive(Component)]
pub struct TriggerZone {
    pub radius: f32,
}

impl TriggerZone {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

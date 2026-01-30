//! Health and damage systems

use bevy::prelude::*;
use bevy::ecs::message::{Message, MessageReader, MessageWriter};

/// Health component for any entity that can take damage
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            maximum: max,
        }
    }

    pub fn fraction(&self) -> f32 {
        self.current / self.maximum
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn heal(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.maximum);
    }
}

/// Damage types for resistance/vulnerability calculations
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect)]
pub enum DamageType {
    Physical,
    Fire,
    Ice,
    Electric,
    Viral,  // For hacker-themed damage
}

impl Default for DamageType {
    fn default() -> Self {
        Self::Physical
    }
}

/// Message fired when damage is dealt
#[derive(Message)]
pub struct DamageEvent {
    pub target: Entity,
    pub source: Option<Entity>,
    pub amount: f32,
    pub damage_type: DamageType,
}

/// Message fired when an entity dies
#[derive(Message)]
pub struct DeathEvent {
    pub entity: Entity,
    pub killed_by: Option<Entity>,
}

/// System that applies damage from DamageEvents
pub fn apply_damage_system(
    mut events: MessageReader<DamageEvent>,
    mut query: Query<&mut Health>,
) {
    for event in events.read() {
        if let Ok(mut health) = query.get_mut(event.target) {
            health.current -= event.amount;
        }
    }
}

/// System that checks for deaths and fires DeathEvents
pub fn check_death_system(
    query: Query<(Entity, &Health), Changed<Health>>,
    mut death_events: MessageWriter<DeathEvent>,
) {
    for (entity, health) in query.iter() {
        if health.is_dead() {
            death_events.write(DeathEvent {
                entity,
                killed_by: None, // Could track last damage source
            });
        }
    }
}

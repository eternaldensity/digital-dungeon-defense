//! Bloom post-processing for neon glow effect

use bevy::prelude::*;
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use crate::MainCamera;

/// Plugin that configures bloom post-processing
pub struct BloomPlugin;

impl Plugin for BloomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, add_bloom_to_camera);
    }
}

fn add_bloom_to_camera(
    mut commands: Commands,
    camera_query: Query<Entity, With<MainCamera>>,
) {
    for camera_entity in camera_query.iter() {
        commands.entity(camera_entity).insert(Bloom {
            // Moderate intensity for visible but not overwhelming glow
            intensity: 0.3,
            // How far the bloom spreads
            low_frequency_boost: 0.5,
            low_frequency_boost_curvature: 0.5,
            // Use prefilter with threshold to only bloom bright areas
            prefilter: BloomPrefilter {
                threshold: 1.0,
                threshold_softness: 0.2,
            },
            // Composite mode for better visual quality
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        });
    }
}

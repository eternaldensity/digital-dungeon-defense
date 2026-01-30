//! Dark ambient lighting with subtle directional light

use bevy::prelude::*;
use crate::TronPalette;

/// Plugin that sets up Tron-style lighting
pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_lighting);
    }
}

fn setup_lighting(mut commands: Commands, palette: Res<TronPalette>) {
    // Set background to pure black
    commands.insert_resource(ClearColor(palette.background));

    // Very dark ambient light with slight blue tint
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.1, 0.1, 0.2),
        brightness: 50.0, // Low brightness for dark atmosphere
        ..default()
    });

    // Subtle directional light from above for form definition
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.6, 0.7, 0.8),
            illuminance: 500.0, // Subtle, not overpowering
            shadows_enabled: false, // Tron aesthetic doesn't use heavy shadows
            ..default()
        },
        Transform::from_xyz(0.0, 100.0, 50.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

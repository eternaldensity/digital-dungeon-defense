//! Factory functions for creating Tron-style materials

use bevy::prelude::*;
use crate::TronPalette;

/// Creates a glassy, emissive material in Tron style
///
/// # Arguments
/// * `color` - Base color for the material
/// * `emissive_intensity` - How bright the glow effect is (1.0-5.0 recommended)
pub fn create_tron_material(color: Color, emissive_intensity: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: color.with_alpha(0.8),
        emissive: TronPalette::to_emissive(color, emissive_intensity),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.1,
        metallic: 0.3,
        reflectance: 0.8,
        ior: 1.5, // Glass-like refraction
        ..default()
    }
}

/// Creates a solid emissive material (no transparency)
pub fn create_solid_tron_material(color: Color, emissive_intensity: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        emissive: TronPalette::to_emissive(color, emissive_intensity),
        perceptual_roughness: 0.2,
        metallic: 0.5,
        reflectance: 0.6,
        ..default()
    }
}

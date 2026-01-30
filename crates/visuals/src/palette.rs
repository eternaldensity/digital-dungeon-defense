//! Tron Legacy color palette

use bevy::prelude::*;

/// Tron Legacy color palette resource
#[derive(Resource)]
pub struct TronPalette {
    /// Cyan glow for player entities (FloofBytes, Core)
    pub cyan: Color,
    /// Orange-red for enemy entities
    pub orange: Color,
    /// Magenta for special abilities (DataSpike)
    pub magenta: Color,
    /// White-cyan for the dungeon core
    pub white_cyan: Color,
    /// Grid line color (darker cyan)
    pub grid: Color,
    /// Firewall orange
    pub firewall: Color,
    /// Background black
    pub background: Color,
}

impl Default for TronPalette {
    fn default() -> Self {
        Self {
            cyan: Color::srgb(0.4, 1.0, 1.0),
            orange: Color::srgb(1.0, 0.5, 0.2),
            magenta: Color::srgb(1.0, 0.4, 1.0),
            white_cyan: Color::srgb(0.8, 1.0, 1.0),
            grid: Color::srgb(0.0, 0.6, 0.8),
            firewall: Color::srgb(1.0, 0.7, 0.3),
            background: Color::BLACK,
        }
    }
}

impl TronPalette {
    /// Convert a palette color to its emissive linear RGB value for materials
    pub fn to_emissive(color: Color, intensity: f32) -> LinearRgba {
        let srgba = color.to_srgba();
        LinearRgba::new(
            srgba.red * intensity,
            srgba.green * intensity,
            srgba.blue * intensity,
            1.0,
        )
    }
}

/// Plugin that initializes the color palette
pub struct PalettePlugin;

impl Plugin for PalettePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TronPalette>();
    }
}

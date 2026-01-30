//! Tron-style materials with glassy emissive properties

mod tron_material;

pub use tron_material::*;

use bevy::prelude::*;
use crate::TronPalette;

/// Pre-created material handles for all entity types
#[derive(Resource)]
pub struct TronMaterials {
    /// Cyan material for player entities (FloofBytes)
    pub cyan: Handle<StandardMaterial>,
    /// Orange-red material for enemies
    pub orange: Handle<StandardMaterial>,
    /// Magenta material for DataSpike
    pub magenta: Handle<StandardMaterial>,
    /// White-cyan material for dungeon core
    pub white_cyan: Handle<StandardMaterial>,
    /// Cyan material for grid lines
    pub grid: Handle<StandardMaterial>,
    /// Orange material for firewall
    pub firewall: Handle<StandardMaterial>,
    /// Dark floor material
    pub floor: Handle<StandardMaterial>,
}

/// Plugin that creates the material resources
pub struct MaterialsPlugin;

impl Plugin for MaterialsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TronPalette>()
            .add_systems(Startup, setup_materials);
    }
}

fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<TronPalette>,
) {
    let tron_materials = TronMaterials {
        cyan: materials.add(create_tron_material(palette.cyan, 3.0)),
        orange: materials.add(create_tron_material(palette.orange, 3.0)),
        magenta: materials.add(create_tron_material(palette.magenta, 3.0)),
        white_cyan: materials.add(create_tron_material(palette.white_cyan, 4.0)),
        grid: materials.add(create_tron_material(palette.grid, 2.0)),
        firewall: materials.add(create_tron_material(palette.firewall, 3.0)),
        floor: materials.add(StandardMaterial {
            base_color: Color::srgb(0.02, 0.02, 0.03),
            perceptual_roughness: 0.9,
            metallic: 0.0,
            reflectance: 0.1,
            ..default()
        }),
    };

    commands.insert_resource(tron_materials);
}

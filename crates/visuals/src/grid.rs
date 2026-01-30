//! Glowing grid floor for Tron aesthetic

use bevy::prelude::*;
use ddd_core::DungeonGrid;
use crate::TronPalette;

/// Marker for grid floor entities
#[derive(Component)]
pub struct GridFloor;

/// Marker for grid line entities
#[derive(Component)]
pub struct GridLine;

/// Plugin that creates the grid floor
pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, setup_grid);
    }
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dungeon: Res<DungeonGrid>,
    palette: Res<TronPalette>,
) {
    let width = dungeon.floor_width as f32 * dungeon.tile_size;
    let height = dungeon.floor_height as f32 * dungeon.tile_size;

    // Create dark floor plane
    let floor_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::new(width, height)));
    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.015, 0.015, 0.02),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        reflectance: 0.05,
        ..default()
    });

    commands.spawn((
        GridFloor,
        Mesh3d(floor_mesh),
        MeshMaterial3d(floor_material),
        Transform::from_xyz(0.0, -1.0, 0.0), // Slightly below entities
    ));

    // Create grid line material (emissive cyan)
    let line_material = materials.add(StandardMaterial {
        base_color: palette.grid.with_alpha(0.6),
        emissive: TronPalette::to_emissive(palette.grid, 1.5),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.2,
        ..default()
    });

    // Grid line dimensions
    let line_thickness = 1.5;
    let line_height = 0.5;

    // Create horizontal grid lines
    let h_line_mesh = meshes.add(Cuboid::new(width, line_height, line_thickness));
    for i in 0..=dungeon.floor_height {
        let z = (i as f32 - dungeon.floor_height as f32 / 2.0) * dungeon.tile_size;
        commands.spawn((
            GridLine,
            Mesh3d(h_line_mesh.clone()),
            MeshMaterial3d(line_material.clone()),
            Transform::from_xyz(0.0, -0.5, z),
        ));
    }

    // Create vertical grid lines
    let v_line_mesh = meshes.add(Cuboid::new(line_thickness, line_height, height));
    for i in 0..=dungeon.floor_width {
        let x = (i as f32 - dungeon.floor_width as f32 / 2.0) * dungeon.tile_size;
        commands.spawn((
            GridLine,
            Mesh3d(v_line_mesh.clone()),
            MeshMaterial3d(line_material.clone()),
            Transform::from_xyz(x, -0.5, 0.0),
        ));
    }
}

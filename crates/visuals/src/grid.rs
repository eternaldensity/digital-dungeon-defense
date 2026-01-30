//! Glowing grid floor for Tron aesthetic

use bevy::prelude::*;
use ddd_core::DungeonGrid;
use ddd_floors::{FireFloor, FloorTypeMarker, IceFloor};
use crate::TronPalette;

/// Marker for grid floor entities
#[derive(Component)]
pub struct GridFloor;

/// Marker for grid line entities
#[derive(Component)]
pub struct GridLine;

/// Resource storing the grid line material handle for dynamic updates
#[derive(Resource)]
pub struct GridLineMaterial(pub Handle<StandardMaterial>);

/// Tracks the current floor type for change detection
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
pub enum ActiveFloorType {
    #[default]
    Normal,
    Fire,
    Ice,
}

/// Plugin that creates the grid floor
pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ActiveFloorType>()
            .add_systems(PostStartup, setup_grid)
            .add_systems(Update, update_grid_color_for_floor_type);
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

    // Create grid line material (emissive cyan - default color)
    let line_material = materials.add(StandardMaterial {
        base_color: palette.grid.with_alpha(0.6),
        emissive: TronPalette::to_emissive(palette.grid, 1.5),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.2,
        ..default()
    });

    // Store material handle for dynamic updates
    commands.insert_resource(GridLineMaterial(line_material.clone()));

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

/// Updates grid line color based on the active floor's type
fn update_grid_color_for_floor_type(
    dungeon: Res<DungeonGrid>,
    fire_floors: Query<&FloorTypeMarker, With<FireFloor>>,
    ice_floors: Query<&FloorTypeMarker, With<IceFloor>>,
    grid_material: Option<Res<GridLineMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<TronPalette>,
    mut current_floor_type: ResMut<ActiveFloorType>,
) {
    let Some(grid_material) = grid_material else {
        return;
    };

    let active_floor = dungeon.active_floor_index;

    // Determine floor type for active floor
    let new_floor_type = if fire_floors.iter().any(|m| m.0 == active_floor) {
        ActiveFloorType::Fire
    } else if ice_floors.iter().any(|m| m.0 == active_floor) {
        ActiveFloorType::Ice
    } else {
        ActiveFloorType::Normal
    };

    // Only update material if floor type changed
    if *current_floor_type != new_floor_type {
        *current_floor_type = new_floor_type;

        if let Some(material) = materials.get_mut(grid_material.0.id()) {
            let color = match new_floor_type {
                ActiveFloorType::Normal => palette.grid,
                ActiveFloorType::Fire => palette.grid_fire,
                ActiveFloorType::Ice => palette.grid_ice,
            };

            material.base_color = color.with_alpha(0.6);
            material.emissive = TronPalette::to_emissive(color, 1.5);
        }
    }
}

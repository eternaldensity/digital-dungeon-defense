//! Glowing grid floor for Tron aesthetic with multi-floor rendering

use bevy::prelude::*;
use ddd_core::DungeonGrid;
use ddd_floors::{FireFloor, FloorTypeMarker, IceFloor};
use crate::TronPalette;

/// Vertical spacing between floor levels
pub const FLOOR_SPACING: f32 = 60.0;
/// Base Y height for floor 0
pub const FLOOR_BASE_Y: f32 = -1.0;

/// Marker for grid floor entities
#[derive(Component)]
pub struct GridFloor;

/// Marker for grid line entities
#[derive(Component)]
pub struct GridLine;

/// Component tracking which floor level a grid element belongs to
#[derive(Component)]
pub struct FloorLayer(pub usize);

/// Resource storing grid line materials per floor for dynamic updates
#[derive(Resource, Default)]
pub struct GridLineMaterials {
    /// Material handles indexed by floor index
    pub materials: Vec<Handle<StandardMaterial>>,
}

/// Tracks the current floor type for change detection per floor
#[derive(Resource, Default)]
pub struct ActiveFloorTypes {
    /// Floor types indexed by floor index
    pub types: Vec<ActiveFloorType>,
}

/// Floor type enum
#[derive(Default, PartialEq, Eq, Clone, Copy)]
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
            .init_resource::<ActiveFloorTypes>()
            .init_resource::<GridLineMaterials>()
            .add_systems(PostStartup, setup_grid)
            .add_systems(Update, (
                update_active_floor_grids,
                update_grid_color_for_floor_type,
            ).chain());
    }
}

fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dungeon: Res<DungeonGrid>,
    palette: Res<TronPalette>,
    mut grid_materials: ResMut<GridLineMaterials>,
    mut floor_types: ResMut<ActiveFloorTypes>,
) {
    let width = dungeon.floor_width as f32 * dungeon.tile_size;
    let height = dungeon.floor_height as f32 * dungeon.tile_size;

    // Create shared meshes
    let floor_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::new(width, height)));
    let line_thickness = 1.5;
    let line_height = 0.5;
    let h_line_mesh = meshes.add(Cuboid::new(width, line_height, line_thickness));
    let v_line_mesh = meshes.add(Cuboid::new(line_thickness, line_height, height));

    // Spawn grid for each floor (up to 3 active floors)
    let num_floors = dungeon.floors.len().max(3);
    for floor_index in 0..num_floors {
        let floor_y = floor_y_position(floor_index);
        let is_active = floor_index == dungeon.active_floor_index;
        let alpha = floor_alpha(floor_index, dungeon.active_floor_index);
        let emissive_mult = floor_emissive_multiplier(floor_index, dungeon.active_floor_index);

        // Dark floor plane
        let floor_material = materials.add(StandardMaterial {
            base_color: Color::srgba(0.015, 0.015, 0.02, alpha * 0.8),
            perceptual_roughness: 0.95,
            metallic: 0.0,
            reflectance: 0.05,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        commands.spawn((
            GridFloor,
            FloorLayer(floor_index),
            Mesh3d(floor_mesh.clone()),
            MeshMaterial3d(floor_material),
            Transform::from_xyz(0.0, floor_y, 0.0),
            Visibility::from(if is_active || adjacent_to_active(floor_index, dungeon.active_floor_index, num_floors) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            }),
        ));

        // Grid line material with depth-based transparency
        let line_material = materials.add(StandardMaterial {
            base_color: palette.grid.with_alpha(alpha),
            emissive: TronPalette::to_emissive(palette.grid, 1.5 * emissive_mult),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.2,
            ..default()
        });
        grid_materials.materials.push(line_material.clone());
        floor_types.types.push(ActiveFloorType::Normal);

        // Horizontal grid lines
        for i in 0..=dungeon.floor_height {
            let z = (i as f32 - dungeon.floor_height as f32 / 2.0) * dungeon.tile_size;
            commands.spawn((
                GridLine,
                FloorLayer(floor_index),
                Mesh3d(h_line_mesh.clone()),
                MeshMaterial3d(line_material.clone()),
                Transform::from_xyz(0.0, floor_y + 0.5, z),
                Visibility::from(if is_active || adjacent_to_active(floor_index, dungeon.active_floor_index, num_floors) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }),
            ));
        }

        // Vertical grid lines
        for i in 0..=dungeon.floor_width {
            let x = (i as f32 - dungeon.floor_width as f32 / 2.0) * dungeon.tile_size;
            commands.spawn((
                GridLine,
                FloorLayer(floor_index),
                Mesh3d(v_line_mesh.clone()),
                MeshMaterial3d(line_material.clone()),
                Transform::from_xyz(x, floor_y + 0.5, 0.0),
                Visibility::from(if is_active || adjacent_to_active(floor_index, dungeon.active_floor_index, num_floors) {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                }),
            ));
        }
    }
}

/// Calculate Y position for a floor based on its index
pub fn floor_y_position(floor_index: usize) -> f32 {
    FLOOR_BASE_Y + (floor_index as f32 * FLOOR_SPACING)
}

/// Calculate alpha transparency based on distance from active floor
fn floor_alpha(floor_index: usize, active_floor: usize) -> f32 {
    let distance = (floor_index as i32 - active_floor as i32).unsigned_abs() as usize;
    match distance {
        0 => 0.7,   // Active floor: high visibility
        1 => 0.3,   // Adjacent floors: reduced visibility
        _ => 0.1,   // Far floors: very faint
    }
}

/// Calculate emissive intensity multiplier based on distance from active floor
fn floor_emissive_multiplier(floor_index: usize, active_floor: usize) -> f32 {
    let distance = (floor_index as i32 - active_floor as i32).unsigned_abs() as usize;
    match distance {
        0 => 1.0,   // Active floor: full glow
        1 => 0.4,   // Adjacent floors: dimmer glow
        _ => 0.15,  // Far floors: minimal glow
    }
}

/// Check if floor is adjacent to active floor
fn adjacent_to_active(floor_index: usize, active_floor: usize, total_floors: usize) -> bool {
    let _ = total_floors; // May use for bounds checking later
    let distance = (floor_index as i32 - active_floor as i32).unsigned_abs() as usize;
    distance == 1
}

/// Update grid visibility and transparency when active floor changes
fn update_active_floor_grids(
    dungeon: Res<DungeonGrid>,
    grid_materials: Res<GridLineMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut floor_query: Query<(&FloorLayer, &mut Visibility, &MeshMaterial3d<StandardMaterial>), With<GridFloor>>,
    mut line_query: Query<(&FloorLayer, &mut Visibility), (With<GridLine>, Without<GridFloor>)>,
    palette: Res<TronPalette>,
) {
    if !dungeon.is_changed() {
        return;
    }

    let active = dungeon.active_floor_index;
    let num_floors = dungeon.floors.len().max(3);

    // Update floor plane visibility and materials
    for (layer, mut visibility, material_handle) in floor_query.iter_mut() {
        let should_show = layer.0 == active || adjacent_to_active(layer.0, active, num_floors);
        *visibility = if should_show { Visibility::Visible } else { Visibility::Hidden };

        // Update floor plane alpha
        if let Some(mat) = materials.get_mut(material_handle.id()) {
            let alpha = floor_alpha(layer.0, active);
            mat.base_color = Color::srgba(0.015, 0.015, 0.02, alpha * 0.8);
        }
    }

    // Update grid line visibility
    for (layer, mut visibility) in line_query.iter_mut() {
        let should_show = layer.0 == active || adjacent_to_active(layer.0, active, num_floors);
        *visibility = if should_show { Visibility::Visible } else { Visibility::Hidden };
    }

    // Update grid line materials (alpha and emissive)
    for (floor_index, mat_handle) in grid_materials.materials.iter().enumerate() {
        if let Some(mat) = materials.get_mut(mat_handle.id()) {
            let alpha = floor_alpha(floor_index, active);
            let emissive_mult = floor_emissive_multiplier(floor_index, active);
            mat.base_color = palette.grid.with_alpha(alpha);
            mat.emissive = TronPalette::to_emissive(palette.grid, 1.5 * emissive_mult);
        }
    }
}

/// Updates grid line color based on each floor's type
fn update_grid_color_for_floor_type(
    dungeon: Res<DungeonGrid>,
    fire_floors: Query<&FloorTypeMarker, With<FireFloor>>,
    ice_floors: Query<&FloorTypeMarker, With<IceFloor>>,
    grid_materials: Res<GridLineMaterials>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<TronPalette>,
    mut floor_types: ResMut<ActiveFloorTypes>,
) {
    let active = dungeon.active_floor_index;

    // Update each floor's grid color based on its type
    for (floor_index, mat_handle) in grid_materials.materials.iter().enumerate() {
        // Determine floor type
        let new_floor_type = if fire_floors.iter().any(|m| m.0 == floor_index) {
            ActiveFloorType::Fire
        } else if ice_floors.iter().any(|m| m.0 == floor_index) {
            ActiveFloorType::Ice
        } else {
            ActiveFloorType::Normal
        };

        // Get previous floor type if available
        let prev_type = floor_types.types.get(floor_index).copied();

        // Only update if type changed
        if prev_type != Some(new_floor_type) {
            if floor_index < floor_types.types.len() {
                floor_types.types[floor_index] = new_floor_type;
            }

            if let Some(material) = materials.get_mut(mat_handle.id()) {
                let color = match new_floor_type {
                    ActiveFloorType::Normal => palette.grid,
                    ActiveFloorType::Fire => palette.grid_fire,
                    ActiveFloorType::Ice => palette.grid_ice,
                };

                let alpha = floor_alpha(floor_index, active);
                let emissive_mult = floor_emissive_multiplier(floor_index, active);

                material.base_color = color.with_alpha(alpha);
                material.emissive = TronPalette::to_emissive(color, 1.5 * emissive_mult);
            }
        }
    }
}

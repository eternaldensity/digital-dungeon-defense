//! Visual rendering for building mode.
//!
//! Renders:
//! - Placed tiles with their walkable patterns
//! - Ghost preview for tile being placed
//! - Planning grid overlay
//! - Construction slot indicators (color-coded by type)
//! - Invalid placement indicator (red tint)
//! - Door positions

use bevy::prelude::*;
use ddd_building::{
    planning_grid_to_world, BuildMode, BuildModeState, DungeonLayout, PlacedTile, SlotType,
    TileOrientation, TileSide, PLANNING_TILE_WORLD_SIZE,
};

use crate::{floor_y_position, TronPalette};

/// Marker for planning grid overlay entities.
#[derive(Component)]
pub struct PlanningGridOverlay;

/// Marker for placed tile visuals.
#[derive(Component)]
pub struct PlacedTileVisual {
    pub floor_index: usize,
    pub position: IVec2,
}

/// Marker for tile cell visuals (walkable pattern).
#[derive(Component)]
pub struct TileCellVisual {
    pub floor_index: usize,
    pub tile_position: IVec2,
}

/// Marker for the ghost preview tile.
#[derive(Component)]
pub struct GhostPreview;

/// Marker for construction slot indicator.
#[derive(Component)]
pub struct SlotIndicator {
    pub floor_index: usize,
    pub tile_position: IVec2,
}

/// Marker for door indicator.
#[derive(Component)]
pub struct DoorIndicator {
    pub floor_index: usize,
    pub tile_position: IVec2,
}

/// Plugin for building mode visuals.
pub struct BuildVisualsPlugin;

impl Plugin for BuildVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_planning_grid_on_build_mode,
                despawn_planning_grid_on_exit,
                sync_placed_tiles,
                update_ghost_preview,
            )
                .run_if(resource_exists::<DungeonLayout>),
        );
    }
}

/// Spawn planning grid overlay when entering build mode.
fn spawn_planning_grid_on_build_mode(
    mut commands: Commands,
    build_mode: Res<State<BuildMode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<TronPalette>,
    existing: Query<Entity, With<PlanningGridOverlay>>,
    build_state: Res<BuildModeState>,
) {
    // Only spawn when in build mode and not already spawned
    if *build_mode.get() == BuildMode::Disabled {
        return;
    }
    if !existing.is_empty() {
        return;
    }

    let floor_y = floor_y_position(build_state.active_floor) + 0.1;

    // Create planning grid lines (larger spacing than combat grid)
    let grid_extent = 5; // Show +/- 5 planning tiles
    let line_length = (grid_extent * 2 + 1) as f32 * PLANNING_TILE_WORLD_SIZE;
    let line_thickness = 2.0;
    let line_height = 0.3;

    let h_line = meshes.add(Cuboid::new(line_length, line_height, line_thickness));
    let v_line = meshes.add(Cuboid::new(line_thickness, line_height, line_length));

    let line_material = materials.add(StandardMaterial {
        base_color: palette.cyan.with_alpha(0.4),
        emissive: TronPalette::to_emissive(palette.cyan, 0.8),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // Spawn horizontal lines
    for i in -grid_extent..=grid_extent {
        let z = i as f32 * PLANNING_TILE_WORLD_SIZE;
        commands.spawn((
            PlanningGridOverlay,
            Mesh3d(h_line.clone()),
            MeshMaterial3d(line_material.clone()),
            Transform::from_xyz(0.0, floor_y, z),
        ));
    }

    // Spawn vertical lines
    for i in -grid_extent..=grid_extent {
        let x = i as f32 * PLANNING_TILE_WORLD_SIZE;
        commands.spawn((
            PlanningGridOverlay,
            Mesh3d(v_line.clone()),
            MeshMaterial3d(line_material.clone()),
            Transform::from_xyz(x, floor_y, 0.0),
        ));
    }
}

/// Despawn planning grid when exiting build mode.
fn despawn_planning_grid_on_exit(
    mut commands: Commands,
    build_mode: Res<State<BuildMode>>,
    query: Query<Entity, With<PlanningGridOverlay>>,
    ghost_query: Query<Entity, With<GhostPreview>>,
    tile_query: Query<Entity, With<PlacedTileVisual>>,
    cell_query: Query<Entity, With<TileCellVisual>>,
    slot_query: Query<Entity, With<SlotIndicator>>,
    door_query: Query<Entity, With<DoorIndicator>>,
) {
    if *build_mode.get() != BuildMode::Disabled {
        return;
    }

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in ghost_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in tile_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in cell_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in slot_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in door_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Sync placed tile visuals with the dungeon layout.
fn sync_placed_tiles(
    mut commands: Commands,
    layout: Res<DungeonLayout>,
    build_mode: Res<State<BuildMode>>,
    build_state: Res<BuildModeState>,
    existing_tiles: Query<(Entity, &PlacedTileVisual)>,
    existing_cells: Query<(Entity, &TileCellVisual)>,
    existing_slots: Query<(Entity, &SlotIndicator)>,
    existing_doors: Query<(Entity, &DoorIndicator)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<TronPalette>,
) {
    if *build_mode.get() == BuildMode::Disabled {
        return;
    }

    if !layout.is_changed() && !build_state.is_changed() {
        return;
    }

    let floor_index = build_state.active_floor;
    let Some(floor) = layout.get_floor(floor_index) else {
        return;
    };

    // Collect existing visual positions
    let mut existing_positions: std::collections::HashSet<IVec2> = std::collections::HashSet::new();

    // Clean up tiles on wrong floor or removed tiles
    for (entity, visual) in existing_tiles.iter() {
        if visual.floor_index != floor_index || !floor.has_tile(visual.position) {
            commands.entity(entity).despawn();
        } else {
            existing_positions.insert(visual.position);
        }
    }

    // Clean up cells, slots, doors for wrong floor or removed tiles
    for (entity, cell) in existing_cells.iter() {
        if cell.floor_index != floor_index || !floor.has_tile(cell.tile_position) {
            commands.entity(entity).despawn();
        }
    }
    for (entity, slot) in existing_slots.iter() {
        if slot.floor_index != floor_index || !floor.has_tile(slot.tile_position) {
            commands.entity(entity).despawn();
        }
    }
    for (entity, door) in existing_doors.iter() {
        if door.floor_index != floor_index || !floor.has_tile(door.tile_position) {
            commands.entity(entity).despawn();
        }
    }

    // Spawn visuals for new tiles
    let floor_y = floor_y_position(floor_index) + 0.15;

    for (pos, tile) in &floor.tiles {
        if existing_positions.contains(pos) {
            continue;
        }

        spawn_tile_visual(
            &mut commands,
            &mut meshes,
            &mut materials,
            &palette,
            &layout,
            tile,
            floor_index,
            floor_y,
        );
    }
}

/// Spawn a visual representation of a placed tile.
fn spawn_tile_visual(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    palette: &TronPalette,
    layout: &DungeonLayout,
    tile: &PlacedTile,
    floor_index: usize,
    floor_y: f32,
) {
    let Some(template) = layout.get_template(tile.template_id) else {
        return;
    };

    let world_pos = planning_grid_to_world(tile.position);
    let center = Vec3::new(world_pos.x, floor_y, world_pos.z);

    // Spawn marker entity for the tile
    commands.spawn((
        PlacedTileVisual {
            floor_index,
            position: tile.position,
        },
        Transform::from_translation(center),
        Visibility::default(),
    ));

    // Tile size in world units (each small tile is 32 units)
    let small_tile_size = 32.0;
    let cell_mesh = meshes.add(Cuboid::new(
        small_tile_size * 0.9,
        0.2,
        small_tile_size * 0.9,
    ));

    let walkable_material = materials.add(StandardMaterial {
        base_color: palette.cyan.with_alpha(0.3),
        emissive: TronPalette::to_emissive(palette.cyan, 0.5),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.1, 0.1, 0.15, 0.6),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // Calculate the offset to center the 9x9 grid
    let half_tile = PLANNING_TILE_WORLD_SIZE / 2.0;
    let half_small = small_tile_size / 2.0;

    // Spawn each cell of the walkable pattern
    for y in 0..9 {
        for x in 0..9 {
            // Apply orientation transform to get the actual pattern index
            let (tx, ty) = transform_pattern_coords(x, y, tile.orientation);
            let walkable = template.walkable_pattern[ty][tx];

            let local_x = (x as f32 * small_tile_size) + half_small - half_tile;
            let local_z = (y as f32 * small_tile_size) + half_small - half_tile;

            let material = if walkable {
                walkable_material.clone()
            } else {
                wall_material.clone()
            };

            commands.spawn((
                TileCellVisual {
                    floor_index,
                    tile_position: tile.position,
                },
                Mesh3d(cell_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(center.x + local_x, floor_y, center.z + local_z),
            ));
        }
    }

    // Spawn slot indicators
    spawn_slot_indicators(
        commands,
        meshes,
        materials,
        palette,
        template,
        tile,
        floor_index,
        center,
        floor_y,
    );

    // Spawn door indicators
    spawn_door_indicators(
        commands,
        meshes,
        materials,
        palette,
        template,
        tile,
        floor_index,
        center,
        floor_y,
    );
}

/// Transform pattern coordinates based on tile orientation.
fn transform_pattern_coords(x: usize, y: usize, orientation: TileOrientation) -> (usize, usize) {
    let x = x as i32;
    let y = y as i32;

    // Apply rotation (0, 90, 180, 270 degrees CW)
    let (rx, ry) = match orientation.rotation {
        0 => (x, y),
        1 => (8 - y, x),      // 90 CW
        2 => (8 - x, 8 - y),  // 180
        3 => (y, 8 - x),      // 270 CW
        _ => (x, y),
    };

    // Apply flip (horizontal flip after rotation)
    let (fx, fy) = if orientation.flipped {
        (8 - rx, ry)
    } else {
        (rx, ry)
    };

    (fx as usize, fy as usize)
}

/// Spawn construction slot indicators for a tile.
fn spawn_slot_indicators(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    palette: &TronPalette,
    template: &ddd_building::TileTemplate,
    tile: &PlacedTile,
    floor_index: usize,
    center: Vec3,
    floor_y: f32,
) {
    let slot_mesh = meshes.add(Sphere::new(8.0).mesh().ico(2).unwrap());

    for slot in &template.slots {
        // Transform slot position based on orientation
        let local_pos = transform_slot_position(slot.local_position, tile.orientation);

        let half_tile = PLANNING_TILE_WORLD_SIZE / 2.0;
        let local_x = local_pos.x * PLANNING_TILE_WORLD_SIZE - half_tile;
        let local_z = local_pos.y * PLANNING_TILE_WORLD_SIZE - half_tile;

        // Color based on slot type
        let color = slot_type_color(slot.slot_type, palette);

        // Check if slot is filled
        let is_filled = !tile.get_feature(slot.slot_id).is_empty();
        let intensity = if is_filled { 2.0 } else { 0.8 };

        let material = materials.add(StandardMaterial {
            base_color: color.with_alpha(0.6),
            emissive: TronPalette::to_emissive(color, intensity),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        commands.spawn((
            SlotIndicator {
                floor_index,
                tile_position: tile.position,
            },
            Mesh3d(slot_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(center.x + local_x, floor_y + 10.0, center.z + local_z),
        ));
    }
}

/// Transform a slot's local position based on tile orientation.
fn transform_slot_position(pos: Vec2, orientation: TileOrientation) -> Vec2 {
    // Rotate around center (0.5, 0.5)
    let centered = pos - Vec2::splat(0.5);

    let rotated = match orientation.rotation {
        0 => centered,
        1 => Vec2::new(-centered.y, centered.x),      // 90 CW
        2 => Vec2::new(-centered.x, -centered.y),     // 180
        3 => Vec2::new(centered.y, -centered.x),      // 270 CW
        _ => centered,
    };

    let flipped = if orientation.flipped {
        Vec2::new(-rotated.x, rotated.y)
    } else {
        rotated
    };

    flipped + Vec2::splat(0.5)
}

/// Get color for a slot type.
fn slot_type_color(slot_type: SlotType, palette: &TronPalette) -> Color {
    match slot_type {
        SlotType::Trap => palette.magenta,
        SlotType::Spawner => palette.orange,
        SlotType::Core => palette.white_cyan,
        SlotType::UpConnector => Color::srgb(0.3, 1.0, 0.3),   // Green
        SlotType::DownConnector => Color::srgb(1.0, 0.3, 0.3), // Red
        SlotType::Decoration => Color::srgb(0.5, 0.5, 0.5),    // Gray
    }
}

/// Spawn door indicators for a tile.
fn spawn_door_indicators(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    palette: &TronPalette,
    template: &ddd_building::TileTemplate,
    tile: &PlacedTile,
    floor_index: usize,
    center: Vec3,
    floor_y: f32,
) {
    let door_mesh = meshes.add(Cuboid::new(20.0, 2.0, 6.0));

    let door_material = materials.add(StandardMaterial {
        base_color: palette.cyan.with_alpha(0.7),
        emissive: TronPalette::to_emissive(palette.cyan, 1.2),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let half_tile = PLANNING_TILE_WORLD_SIZE / 2.0;
    let small_tile = 32.0;

    for door in &template.doors {
        // Transform door based on orientation
        let transformed_door = transform_door(door, tile.orientation);

        // Calculate door position
        let door_idx = transformed_door.position.index() as f32;
        let offset_along_edge = (door_idx * small_tile) + (small_tile / 2.0) - half_tile;

        let (local_x, local_z, rotation) = match transformed_door.side {
            TileSide::North => (offset_along_edge, half_tile, 0.0),
            TileSide::South => (offset_along_edge, -half_tile, 0.0),
            TileSide::East => (half_tile, offset_along_edge, std::f32::consts::FRAC_PI_2),
            TileSide::West => (-half_tile, offset_along_edge, std::f32::consts::FRAC_PI_2),
        };

        commands.spawn((
            DoorIndicator {
                floor_index,
                tile_position: tile.position,
            },
            Mesh3d(door_mesh.clone()),
            MeshMaterial3d(door_material.clone()),
            Transform::from_xyz(center.x + local_x, floor_y + 1.0, center.z + local_z)
                .with_rotation(Quat::from_rotation_y(rotation)),
        ));
    }
}

/// Transform a door ID based on tile orientation.
fn transform_door(door: &ddd_building::DoorId, orientation: TileOrientation) -> ddd_building::DoorId {
    use ddd_building::{DoorId, DoorPosition};

    let mut side = door.side;
    let mut position = door.position;

    // Apply rotation
    for _ in 0..orientation.rotation {
        side = side.rotate_cw();
    }

    // Apply flip (mirror horizontally)
    if orientation.flipped {
        side = match side {
            TileSide::North => TileSide::North,
            TileSide::South => TileSide::South,
            TileSide::East => TileSide::West,
            TileSide::West => TileSide::East,
        };
        // Also mirror the position along the edge
        position = match position {
            DoorPosition::Left => DoorPosition::Right,
            DoorPosition::Center => DoorPosition::Center,
            DoorPosition::Right => DoorPosition::Left,
        };
    }

    DoorId::new(side, position)
}

/// Update the ghost preview tile.
fn update_ghost_preview(
    mut commands: Commands,
    build_mode: Res<State<BuildMode>>,
    build_state: Res<BuildModeState>,
    layout: Res<DungeonLayout>,
    existing_ghost: Query<Entity, With<GhostPreview>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    palette: Res<TronPalette>,
) {
    // Despawn existing ghost
    for entity in existing_ghost.iter() {
        commands.entity(entity).despawn();
    }

    // Only show ghost in TilePlace mode with a selected template
    if *build_mode.get() != BuildMode::TilePlace {
        return;
    }

    let Some(template_id) = build_state.selected_template else {
        return;
    };

    let Some(position) = build_state.preview_position else {
        return;
    };

    let Some(template) = layout.get_template(template_id) else {
        return;
    };

    let floor_index = build_state.active_floor;
    let Some(floor) = layout.get_floor(floor_index) else {
        return;
    };

    let floor_y = floor_y_position(floor_index) + 0.2;

    // Check if placement is valid
    let is_valid = ddd_building::can_place_tile(
        floor,
        &layout.templates,
        position,
        template_id,
        build_state.current_orientation,
    );

    // Spawn ghost visual
    spawn_ghost_tile_visual(
        &mut commands,
        &mut meshes,
        &mut materials,
        &palette,
        template,
        position,
        build_state.current_orientation,
        floor_y,
        is_valid,
    );
}

/// Spawn a ghost preview tile visual.
fn spawn_ghost_tile_visual(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    palette: &TronPalette,
    template: &ddd_building::TileTemplate,
    position: IVec2,
    orientation: TileOrientation,
    floor_y: f32,
    is_valid: bool,
) {
    let world_pos = planning_grid_to_world(position);
    let center = Vec3::new(world_pos.x, floor_y, world_pos.z);

    let small_tile_size = 32.0;
    let cell_mesh = meshes.add(Cuboid::new(
        small_tile_size * 0.85,
        0.15,
        small_tile_size * 0.85,
    ));

    // Color based on validity
    let (walkable_color, wall_color) = if is_valid {
        (palette.cyan, Color::srgba(0.1, 0.1, 0.15, 0.3))
    } else {
        (Color::srgb(1.0, 0.2, 0.2), Color::srgba(0.3, 0.1, 0.1, 0.3))
    };

    let ghost_walkable = materials.add(StandardMaterial {
        base_color: walkable_color.with_alpha(0.2),
        emissive: TronPalette::to_emissive(walkable_color, 0.3),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let ghost_wall = materials.add(StandardMaterial {
        base_color: wall_color,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let half_tile = PLANNING_TILE_WORLD_SIZE / 2.0;
    let half_small = small_tile_size / 2.0;

    for y in 0..9 {
        for x in 0..9 {
            let (tx, ty) = transform_pattern_coords(x, y, orientation);
            let walkable = template.walkable_pattern[ty][tx];

            let local_x = (x as f32 * small_tile_size) + half_small - half_tile;
            let local_z = (y as f32 * small_tile_size) + half_small - half_tile;

            let material = if walkable {
                ghost_walkable.clone()
            } else {
                ghost_wall.clone()
            };

            commands.spawn((
                GhostPreview,
                Mesh3d(cell_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(center.x + local_x, floor_y, center.z + local_z),
            ));
        }
    }
}

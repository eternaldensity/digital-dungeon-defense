//! Layout compiler - converts planning tiles to gameplay grid and spawns entities.
//!
//! The planning system uses large 9x9 planning tiles for dungeon construction.
//! This module compiles those into the smaller gameplay grid used for pathfinding
//! and combat, and spawns the actual game entities from placed features.

use bevy::prelude::*;
use ddd_building::{
    planning_grid_to_world, DungeonLayout, PlacedFeature, SlotType, TileOrientation, TileSide,
    PLANNING_TILE_SIZE, PLANNING_TILE_WORLD_SIZE,
};
use ddd_core::{DungeonCore, DungeonGrid, Floor, GridPosition, Health, OnFloor, Team, TileType};
use ddd_creatures::BasicFloofSpawnerBundle;
use ddd_traps::{DataSpikeBundle, FirewallBundle};

/// Size of a small tile in world units.
pub const SMALL_TILE_SIZE: f32 = 32.0;

/// Result of compiling a layout.
#[derive(Default)]
pub struct CompilationResult {
    /// Width of the compiled grid in small tiles.
    pub width: usize,
    /// Height of the compiled grid in small tiles.
    pub height: usize,
    /// Number of floors compiled.
    pub floor_count: usize,
    /// Enemy spawn points (world position, floor index).
    pub enemy_spawn_points: Vec<(Vec3, usize)>,
}

/// Entities spawned from the compiled layout.
#[derive(Default)]
pub struct SpawnedEntities {
    pub core: Option<Entity>,
    pub traps: Vec<Entity>,
    pub spawners: Vec<Entity>,
}

/// Compiles a DungeonLayout into a DungeonGrid for gameplay.
///
/// Each planning tile (9x9) is expanded into the gameplay grid. The walkable
/// pattern from each template is applied with the tile's orientation transform.
pub fn compile_layout(layout: &DungeonLayout, grid: &mut DungeonGrid) -> CompilationResult {
    let mut result = CompilationResult::default();

    // Get bounds across all floors
    let Some((min, max)) = layout.bounds() else {
        // Empty layout - use minimal grid
        *grid = DungeonGrid::default();
        return result;
    };

    // Calculate grid dimensions (each planning tile = 9x9 small tiles)
    let planning_width = (max.x - min.x + 1) as usize;
    let planning_height = (max.y - min.y + 1) as usize;
    let grid_width = planning_width * PLANNING_TILE_SIZE;
    let grid_height = planning_height * PLANNING_TILE_SIZE;

    result.width = grid_width;
    result.height = grid_height;
    result.floor_count = layout.floors.len();

    // Resize and clear the gameplay grid
    grid.floor_width = grid_width;
    grid.floor_height = grid_height;
    grid.floors.clear();

    // Compile each floor
    for (floor_idx, planning_floor) in layout.floors.iter().enumerate() {
        let mut floor = Floor::new(floor_idx);
        floor.tiles = vec![vec![TileType::Wall; grid_width]; grid_height];

        for (planning_pos, placed_tile) in &planning_floor.tiles {
            let Some(template) = layout.get_template(placed_tile.template_id) else {
                continue;
            };

            // Calculate base position in the gameplay grid
            // Offset so planning_pos min maps to (0, 0)
            let base_x = (planning_pos.x - min.x) as usize * PLANNING_TILE_SIZE;
            let base_y = (planning_pos.y - min.y) as usize * PLANNING_TILE_SIZE;

            // Apply walkable pattern with orientation transform
            for local_y in 0..PLANNING_TILE_SIZE {
                for local_x in 0..PLANNING_TILE_SIZE {
                    // Transform coordinates according to tile orientation
                    let (tx, ty) = placed_tile
                        .orientation
                        .transform_local_coords(local_x, local_y);

                    let walkable = template.walkable_pattern[ty][tx];
                    let grid_x = base_x + local_x;
                    let grid_y = base_y + local_y;

                    if grid_x < grid_width && grid_y < grid_height {
                        floor.tiles[grid_y][grid_x] =
                            if walkable { TileType::Path } else { TileType::Wall };
                    }
                }
            }

            // Mark feature positions in the grid
            for slot in &template.slots {
                // Transform slot position according to orientation
                let transformed_pos = placed_tile
                    .orientation
                    .transform_local_position(slot.local_position);

                // Convert normalized position (0-1) to local tile indices
                let slot_local_x =
                    (transformed_pos.x * (PLANNING_TILE_SIZE - 1) as f32).round() as usize;
                let slot_local_y =
                    (transformed_pos.y * (PLANNING_TILE_SIZE - 1) as f32).round() as usize;

                let grid_x = base_x + slot_local_x;
                let grid_y = base_y + slot_local_y;

                if grid_x >= grid_width || grid_y >= grid_height {
                    continue;
                }

                // Check if there's a feature placed in this slot
                let feature = placed_tile.get_feature(slot.slot_id);

                let tile_type = match feature {
                    PlacedFeature::Trap { .. } => TileType::Trap,
                    PlacedFeature::Spawner { .. } => TileType::Spawner,
                    PlacedFeature::Core => TileType::Core,
                    _ => {
                        // Mark slot position based on slot type for empty slots
                        match slot.slot_type {
                            SlotType::Trap => continue, // Don't mark empty trap slots
                            SlotType::Spawner => continue,
                            SlotType::Core => TileType::Core, // Core slot is always marked
                            SlotType::UpConnector | SlotType::DownConnector => TileType::Path,
                            SlotType::Decoration => continue,
                        }
                    }
                };

                floor.tiles[grid_y][grid_x] = tile_type;
            }
        }

        grid.floors.push(floor);
    }

    // Calculate enemy spawn points (up connectors on top floor)
    result.enemy_spawn_points = calculate_enemy_spawn_points(layout, min);

    result
}

/// Calculate positions where enemies should spawn.
///
/// Enemies enter through UpConnector slots on the top floor and try to reach
/// the core by moving downward through the dungeon.
fn calculate_enemy_spawn_points(layout: &DungeonLayout, min_bounds: IVec2) -> Vec<(Vec3, usize)> {
    let mut spawn_points = Vec::new();

    // Find the top floor (highest index with tiles)
    let top_floor_idx = layout
        .floors
        .iter()
        .enumerate()
        .rev()
        .find(|(_, floor)| !floor.tiles.is_empty())
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    let Some(top_floor) = layout.floors.get(top_floor_idx) else {
        return spawn_points;
    };

    // Find all UpConnector slots on the top floor
    for (planning_pos, placed_tile) in &top_floor.tiles {
        let Some(template) = layout.get_template(placed_tile.template_id) else {
            continue;
        };

        for slot in &template.slots {
            // Look for UpConnector slots (entrance points for enemies)
            if matches!(slot.slot_type, SlotType::UpConnector) {
                let world_pos = slot_to_world_position(
                    *planning_pos,
                    min_bounds,
                    slot.local_position,
                    placed_tile.orientation,
                );
                spawn_points.push((world_pos, top_floor_idx));
            }
        }
    }

    // If no up connectors found, fall back to any door leading to void on top floor
    if spawn_points.is_empty() {
        for (planning_pos, placed_tile) in &top_floor.tiles {
            let Some(template) = layout.get_template(placed_tile.template_id) else {
                continue;
            };

            for door in &template.doors {
                let transformed_door = placed_tile.orientation.transform_door(*door);
                let neighbor_pos = *planning_pos + transformed_door.side.direction();

                if !top_floor.has_tile(neighbor_pos) {
                    let world_pos = door_to_world_position(
                        *planning_pos,
                        min_bounds,
                        transformed_door.side,
                        transformed_door.position.index(),
                    );
                    spawn_points.push((world_pos, top_floor_idx));
                }
            }
        }
    }

    spawn_points
}

/// Convert a door position to world coordinates.
fn door_to_world_position(
    planning_pos: IVec2,
    min_bounds: IVec2,
    side: TileSide,
    door_index: usize,
) -> Vec3 {
    // Base position of the planning tile (center)
    let tile_center = planning_grid_to_world(planning_pos);

    // Offset to the edge based on door side
    let half_size = PLANNING_TILE_WORLD_SIZE / 2.0;
    let door_offset = (door_index as f32 - 4.0) * SMALL_TILE_SIZE; // -4 to center around middle

    let (edge_x, edge_z) = match side {
        TileSide::North => (door_offset, half_size),
        TileSide::South => (door_offset, -half_size),
        TileSide::East => (half_size, door_offset),
        TileSide::West => (-half_size, door_offset),
    };

    // Offset the world position so the grid starts at origin
    let origin_offset = Vec3::new(
        min_bounds.x as f32 * PLANNING_TILE_WORLD_SIZE,
        0.0,
        min_bounds.y as f32 * PLANNING_TILE_WORLD_SIZE,
    );

    Vec3::new(
        tile_center.x + edge_x - origin_offset.x,
        0.0,
        tile_center.z + edge_z - origin_offset.z,
    )
}

/// Spawns gameplay entities based on the compiled layout.
///
/// Creates actual Bevy entities for traps, spawners, and the dungeon core
/// at the positions defined by the planning layout.
pub fn spawn_gameplay_entities(
    commands: &mut Commands,
    layout: &DungeonLayout,
    grid: &DungeonGrid,
) -> SpawnedEntities {
    let mut result = SpawnedEntities::default();

    let Some((min, _max)) = layout.bounds() else {
        return result;
    };

    for (floor_idx, planning_floor) in layout.floors.iter().enumerate() {
        for (planning_pos, placed_tile) in &planning_floor.tiles {
            let Some(template) = layout.get_template(placed_tile.template_id) else {
                continue;
            };

            for slot in &template.slots {
                let feature = placed_tile.get_feature(slot.slot_id);

                // Calculate world position for this slot
                let world_pos = slot_to_world_position(
                    *planning_pos,
                    min,
                    slot.local_position,
                    placed_tile.orientation,
                );

                match feature {
                    PlacedFeature::Trap { trap_type } => {
                        use ddd_building::tiles::TrapType;
                        let entity = match trap_type {
                            TrapType::Firewall => {
                                commands.spawn(FirewallBundle::new(world_pos, floor_idx))
                            }
                            TrapType::DataSpike => {
                                commands.spawn(DataSpikeBundle::new(world_pos, floor_idx))
                            }
                        };
                        result.traps.push(entity.id());
                    }
                    PlacedFeature::Spawner { .. } => {
                        let entity =
                            commands.spawn(BasicFloofSpawnerBundle::new(world_pos, floor_idx));
                        result.spawners.push(entity.id());
                    }
                    PlacedFeature::Core => {
                        // Spawn dungeon core
                        let grid_pos = world_to_grid_position(world_pos, grid, floor_idx);
                        let entity = commands
                            .spawn((
                                DungeonCore,
                                Health::new(100.0),
                                Team::Player,
                                grid_pos,
                                OnFloor(floor_idx),
                                Transform::from_translation(world_pos),
                                Sprite {
                                    color: Color::srgb(0.2, 0.8, 1.0),
                                    custom_size: Some(Vec2::splat(48.0)),
                                    ..default()
                                },
                            ))
                            .id();
                        result.core = Some(entity);
                    }
                    _ => {
                        // Handle empty Core slots - spawn core at the slot position
                        if matches!(slot.slot_type, SlotType::Core) && result.core.is_none() {
                            let grid_pos = world_to_grid_position(world_pos, grid, floor_idx);
                            let entity = commands
                                .spawn((
                                    DungeonCore,
                                    Health::new(100.0),
                                    Team::Player,
                                    grid_pos,
                                    OnFloor(floor_idx),
                                    Transform::from_translation(world_pos),
                                    Sprite {
                                        color: Color::srgb(0.2, 0.8, 1.0),
                                        custom_size: Some(Vec2::splat(48.0)),
                                        ..default()
                                    },
                                ))
                                .id();
                            result.core = Some(entity);
                        }
                    }
                }
            }
        }
    }

    result
}

/// Convert a construction slot to world position.
fn slot_to_world_position(
    planning_pos: IVec2,
    min_bounds: IVec2,
    local_pos: Vec2,
    orientation: TileOrientation,
) -> Vec3 {
    // Transform the local position according to orientation
    let transformed = orientation.transform_local_position(local_pos);

    // Base position of the planning tile (corner)
    let tile_base = Vec3::new(
        planning_pos.x as f32 * PLANNING_TILE_WORLD_SIZE,
        0.0,
        planning_pos.y as f32 * PLANNING_TILE_WORLD_SIZE,
    );

    // Add slot offset within the tile
    let slot_offset = Vec3::new(
        transformed.x * PLANNING_TILE_WORLD_SIZE,
        0.0,
        transformed.y * PLANNING_TILE_WORLD_SIZE,
    );

    // Origin offset so grid starts near (0, 0)
    let origin_offset = Vec3::new(
        min_bounds.x as f32 * PLANNING_TILE_WORLD_SIZE,
        0.0,
        min_bounds.y as f32 * PLANNING_TILE_WORLD_SIZE,
    );

    tile_base + slot_offset - origin_offset
}

/// Convert world position to grid position.
fn world_to_grid_position(world: Vec3, grid: &DungeonGrid, floor: usize) -> GridPosition {
    let x = (world.x / SMALL_TILE_SIZE).floor() as usize;
    let y = (world.z / SMALL_TILE_SIZE).floor() as usize;
    GridPosition::new(
        x.min(grid.floor_width.saturating_sub(1)),
        y.min(grid.floor_height.saturating_sub(1)),
        floor,
    )
}

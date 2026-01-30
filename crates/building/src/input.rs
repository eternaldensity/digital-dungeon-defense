//! Input handling for building mode.
//!
//! Handles keyboard and mouse input during the building phase:
//! - B: Toggle build mode
//! - R: Rotate selected tile
//! - F: Flip selected tile
//! - Left click: Place tile / select tile
//! - Right click: Remove tile / cancel selection
//! - Space/Enter: Exit build mode and start wave
//! - 1-7: Select tile template directly
//! - Tab: Cycle through floors

use crate::{
    can_place_tile, validate_dungeon, BuildMode, BuildModeState, DungeonLayout, PlacedTile,
    TileOrientation, TileTemplateId, PLANNING_TILE_SIZE,
};
use bevy::ecs::message::{Message, MessageWriter};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// World units per planning tile (9 small tiles * 32 units each).
pub const PLANNING_TILE_WORLD_SIZE: f32 = (PLANNING_TILE_SIZE as f32) * 32.0;

/// Plugin for building mode input.
pub struct BuildInputPlugin;

impl Plugin for BuildInputPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register messages
            .add_message::<TilePlacedEvent>()
            .add_message::<TileRemovedEvent>()
            .add_message::<BuildModeChangedEvent>()
            // Input systems
            .add_systems(
                Update,
                (
                    toggle_build_mode,
                    handle_tile_selection,
                    handle_tile_rotation,
                    handle_tile_placement,
                    handle_tile_removal,
                    update_preview_position,
                    handle_floor_switching,
                    exit_build_mode,
                ),
            );
    }
}

/// Message fired when a tile is placed.
#[derive(Message)]
pub struct TilePlacedEvent {
    pub floor: usize,
    pub position: IVec2,
    pub template_id: TileTemplateId,
}

/// Message fired when a tile is removed.
#[derive(Message)]
pub struct TileRemovedEvent {
    pub floor: usize,
    pub position: IVec2,
}

/// Message fired when build mode changes.
#[derive(Message)]
pub struct BuildModeChangedEvent {
    pub enabled: bool,
}

/// Toggle build mode with B key.
fn toggle_build_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<BuildMode>>,
    mut next_state: ResMut<NextState<BuildMode>>,
    mut build_state: ResMut<BuildModeState>,
    mut events: MessageWriter<BuildModeChangedEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        if *current_state.get() == BuildMode::Disabled {
            next_state.set(BuildMode::View);
            build_state.clear_selection();
            events.write(BuildModeChangedEvent { enabled: true });
        } else {
            next_state.set(BuildMode::Disabled);
            build_state.clear_selection();
            events.write(BuildModeChangedEvent { enabled: false });
        }
    }
}

/// Handle direct tile template selection with number keys and numpad.
fn handle_tile_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<BuildMode>>,
    mut build_state: ResMut<BuildModeState>,
    mut next_mode: ResMut<NextState<BuildMode>>,
    layout: Res<DungeonLayout>,
) {
    // Only run in build mode
    if *current_state.get() == BuildMode::Disabled {
        return;
    }

    // Map both number row and numpad keys to templates
    let key_to_template = [
        // Number row
        (KeyCode::Digit1, TileTemplateId(1)), // Basic Room
        (KeyCode::Digit2, TileTemplateId(2)), // Corridor
        (KeyCode::Digit3, TileTemplateId(3)), // Core Chamber
        (KeyCode::Digit4, TileTemplateId(4)), // Stairwell
        (KeyCode::Digit5, TileTemplateId(5)), // L-Room
        (KeyCode::Digit6, TileTemplateId(6)), // T-Junction
        (KeyCode::Digit7, TileTemplateId(7)), // Corner
        // Numpad
        (KeyCode::Numpad1, TileTemplateId(1)),
        (KeyCode::Numpad2, TileTemplateId(2)),
        (KeyCode::Numpad3, TileTemplateId(3)),
        (KeyCode::Numpad4, TileTemplateId(4)),
        (KeyCode::Numpad5, TileTemplateId(5)),
        (KeyCode::Numpad6, TileTemplateId(6)),
        (KeyCode::Numpad7, TileTemplateId(7)),
    ];

    for (key, template_id) in key_to_template {
        if keyboard.just_pressed(key) {
            // Verify template exists
            if layout.get_template(template_id).is_some() {
                build_state.selected_template = Some(template_id);
                build_state.current_orientation = TileOrientation::default();
                next_mode.set(BuildMode::TilePlace);
            }
        }
    }

    // Escape to deselect
    if keyboard.just_pressed(KeyCode::Escape) {
        build_state.selected_template = None;
        build_state.preview_position = None;
        next_mode.set(BuildMode::View);
    }
}

/// Handle tile rotation with R key and flip with F key.
fn handle_tile_rotation(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<BuildMode>>,
    mut build_state: ResMut<BuildModeState>,
) {
    // Only run in build mode
    if *current_state.get() == BuildMode::Disabled {
        return;
    }

    if build_state.selected_template.is_some() {
        if keyboard.just_pressed(KeyCode::KeyR) {
            build_state.current_orientation.rotate_cw();
        }
        if keyboard.just_pressed(KeyCode::KeyF) {
            build_state.current_orientation.flip();
        }
    }
}

/// Handle tile placement with left click.
fn handle_tile_placement(
    mouse: Res<ButtonInput<MouseButton>>,
    current_state: Res<State<BuildMode>>,
    build_state: Res<BuildModeState>,
    mut layout: ResMut<DungeonLayout>,
    mut events: MessageWriter<TilePlacedEvent>,
) {
    // Only run in build mode
    if *current_state.get() == BuildMode::Disabled {
        return;
    }

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(template_id) = build_state.selected_template else {
        return;
    };

    let Some(position) = build_state.preview_position else {
        return;
    };

    let floor_index = build_state.active_floor;

    // Check if we can place here
    let Some(floor) = layout.get_floor(floor_index) else {
        return;
    };

    if !can_place_tile(
        floor,
        &layout.templates,
        position,
        template_id,
        build_state.current_orientation,
    ) {
        return;
    }

    // Create and place the tile
    let mut tile = PlacedTile::new(template_id, position);
    tile.orientation = build_state.current_orientation;

    if let Some(floor) = layout.get_floor_mut(floor_index) {
        floor.place_tile(position, tile);

        // Re-validate the dungeon
        let all_errors = validate_dungeon(&layout);
        // Extract errors for this floor
        let floor_errors: Vec<_> = all_errors
            .into_iter()
            .filter(|(idx, _)| *idx == floor_index)
            .flat_map(|(_, errs)| errs)
            .collect();

        if let Some(floor) = layout.get_floor_mut(floor_index) {
            floor.set_validation_result(floor_errors);
        }

        events.write(TilePlacedEvent {
            floor: floor_index,
            position,
            template_id,
        });
    }
}

/// Handle tile removal with right click.
fn handle_tile_removal(
    mouse: Res<ButtonInput<MouseButton>>,
    current_state: Res<State<BuildMode>>,
    build_state: Res<BuildModeState>,
    mut layout: ResMut<DungeonLayout>,
    mut events: MessageWriter<TileRemovedEvent>,
) {
    // Only run in build mode
    if *current_state.get() == BuildMode::Disabled {
        return;
    }

    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(position) = build_state.preview_position else {
        return;
    };

    let floor_index = build_state.active_floor;

    if let Some(floor) = layout.get_floor_mut(floor_index) {
        if floor.remove_tile(position).is_some() {
            events.write(TileRemovedEvent {
                floor: floor_index,
                position,
            });
        }
    }
}

/// Update the preview position based on mouse cursor.
fn update_preview_position(
    current_state: Res<State<BuildMode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut build_state: ResMut<BuildModeState>,
) {
    // Only run in build mode
    if *current_state.get() == BuildMode::Disabled {
        return;
    }

    let Ok(window) = window_query.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        build_state.preview_position = None;
        return;
    };

    // Find the camera (assuming single camera setup)
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    // Convert screen position to world position
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
        build_state.preview_position = None;
        return;
    };

    // Project ray onto the XZ plane (y = 0)
    // Ray: origin + t * direction, solve for t where y = 0
    if ray.direction.y.abs() < 0.0001 {
        // Ray is parallel to ground, no intersection
        build_state.preview_position = None;
        return;
    }

    let t = -ray.origin.y / ray.direction.y;
    if t < 0.0 {
        // Intersection behind camera
        build_state.preview_position = None;
        return;
    }

    let world_pos = ray.origin + t * ray.direction;

    // Convert world position to planning grid position
    let grid_x = (world_pos.x / PLANNING_TILE_WORLD_SIZE).floor() as i32;
    let grid_y = (world_pos.z / PLANNING_TILE_WORLD_SIZE).floor() as i32;

    build_state.preview_position = Some(IVec2::new(grid_x, grid_y));
}

/// Handle floor switching with Tab key.
fn handle_floor_switching(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<BuildMode>>,
    mut build_state: ResMut<BuildModeState>,
    layout: Res<DungeonLayout>,
) {
    // Only run in build mode
    if *current_state.get() == BuildMode::Disabled {
        return;
    }

    if keyboard.just_pressed(KeyCode::Tab) {
        let shift_held =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
        let floor_count = layout.floor_count();
        let current = build_state.active_floor;

        if shift_held {
            // Go to previous floor
            if current > 0 {
                build_state.set_active_floor(current - 1);
            }
        } else {
            // Go to next floor
            if current + 1 < floor_count {
                build_state.set_active_floor(current + 1);
            }
        }
    }

    // PageUp/PageDown for floor navigation
    let current = build_state.active_floor;
    let floor_count = layout.floor_count();
    if keyboard.just_pressed(KeyCode::PageUp) && current + 1 < floor_count {
        build_state.set_active_floor(current + 1);
    }
    if keyboard.just_pressed(KeyCode::PageDown) && current > 0 {
        build_state.set_active_floor(current - 1);
    }
}

/// Exit build mode with Space or Enter.
fn exit_build_mode(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<BuildMode>>,
    mut next_state: ResMut<NextState<BuildMode>>,
    mut build_state: ResMut<BuildModeState>,
    mut events: MessageWriter<BuildModeChangedEvent>,
    layout: Res<DungeonLayout>,
) {
    // Only run in build mode
    if *current_state.get() == BuildMode::Disabled {
        return;
    }

    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        // Only allow exit if dungeon is valid
        if layout.is_valid() {
            next_state.set(BuildMode::Disabled);
            build_state.clear_selection();
            events.write(BuildModeChangedEvent { enabled: false });
        }
    }
}

/// Convert a planning grid position to world coordinates (center of tile).
pub fn planning_grid_to_world(grid_pos: IVec2) -> Vec3 {
    let half_tile = PLANNING_TILE_WORLD_SIZE / 2.0;
    Vec3::new(
        grid_pos.x as f32 * PLANNING_TILE_WORLD_SIZE + half_tile,
        0.0,
        grid_pos.y as f32 * PLANNING_TILE_WORLD_SIZE + half_tile,
    )
}

/// Convert world coordinates to planning grid position.
pub fn world_to_planning_grid(world_pos: Vec3) -> IVec2 {
    IVec2::new(
        (world_pos.x / PLANNING_TILE_WORLD_SIZE).floor() as i32,
        (world_pos.z / PLANNING_TILE_WORLD_SIZE).floor() as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planning_grid_to_world() {
        let world = planning_grid_to_world(IVec2::ZERO);
        assert_eq!(world.x, PLANNING_TILE_WORLD_SIZE / 2.0);
        assert_eq!(world.z, PLANNING_TILE_WORLD_SIZE / 2.0);
        assert_eq!(world.y, 0.0);
    }

    #[test]
    fn test_world_to_planning_grid() {
        let grid = world_to_planning_grid(Vec3::new(100.0, 0.0, 100.0));
        // 100 / 288 = 0.347..., floor = 0
        assert_eq!(grid, IVec2::ZERO);

        let grid2 = world_to_planning_grid(Vec3::new(300.0, 0.0, 300.0));
        // 300 / 288 = 1.04..., floor = 1
        assert_eq!(grid2, IVec2::ONE);
    }

    #[test]
    fn test_round_trip_conversion() {
        for x in -3..=3 {
            for y in -3..=3 {
                let original = IVec2::new(x, y);
                let world = planning_grid_to_world(original);
                let back = world_to_planning_grid(world);
                assert_eq!(original, back, "Round trip failed for {:?}", original);
            }
        }
    }
}

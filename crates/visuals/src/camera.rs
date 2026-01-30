//! 3D perspective camera with angled isometric-style view
//!
//! Controls:
//! - Arrow keys: Pan camera target
//! - Mouse wheel: Zoom in/out

use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use ddd_building::{world_bounds_for_floor, BuildMode, BuildModeState};

/// Marker component for the main game camera
#[derive(Component)]
pub struct MainCamera;

/// Camera settings for the isometric-style 3D view
#[derive(Resource)]
pub struct CameraSettings {
    /// Distance from the camera to the look-at target
    pub distance: f32,
    /// Pitch angle in degrees (angle down from horizontal, 0 = looking at horizon, 90 = looking straight down)
    pub pitch_degrees: f32,
    /// Yaw angle in degrees (rotation around Y axis, 0 = looking along -Z)
    pub yaw_degrees: f32,
    /// Vertical field of view in degrees
    pub fov_degrees: f32,
    /// Pan speed in world units per second
    pub pan_speed: f32,
    /// Zoom speed multiplier
    pub zoom_speed: f32,
    /// Minimum zoom distance
    pub min_distance: f32,
    /// Maximum zoom distance
    pub max_distance: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            distance: 400.0,
            pitch_degrees: 55.0,  // Angled down to see 3D depth
            yaw_degrees: 0.0,     // Looking along -Z (north in game terms)
            fov_degrees: 50.0,
            pan_speed: 400.0,
            zoom_speed: 50.0,
            min_distance: 100.0,
            max_distance: 800.0,
        }
    }
}

/// Tracks current and target camera state for smooth interpolation
#[derive(Resource)]
pub struct CameraState {
    /// Current look-at target (interpolates toward target_look_at)
    pub current_look_at: Vec3,
    /// Desired look-at target
    pub target_look_at: Vec3,
    /// Current zoom distance (interpolates toward target_distance)
    pub current_distance: f32,
    /// Desired zoom distance
    pub target_distance: f32,
    /// Smoothing factor for position (higher = faster)
    pub smoothing: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            current_look_at: Vec3::ZERO,
            target_look_at: Vec3::ZERO,
            current_distance: 400.0,
            target_distance: 400.0,
            smoothing: 5.0,
        }
    }
}

/// Plugin that sets up the 3D perspective camera
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSettings::default())
            .insert_resource(CameraState::default())
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (
                handle_camera_pan_input,
                handle_camera_zoom_input,
                clamp_camera_to_bounds,
                update_camera_smooth,
            ).chain());
    }
}

fn setup_camera(mut commands: Commands, settings: Res<CameraSettings>, state: Res<CameraState>) {
    let (position, rotation) = calculate_camera_transform(settings.distance, &settings, state.current_look_at);

    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: settings.fov_degrees.to_radians(),
            near: 1.0,
            far: 2000.0,
            ..default()
        }),
        Transform {
            translation: position,
            rotation,
            ..default()
        },
    ));
}

/// Handle arrow key and WASD input to pan camera target
fn handle_camera_pan_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    settings: Res<CameraSettings>,
    mut state: ResMut<CameraState>,
    time: Res<Time>,
) {
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        direction.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        direction.z += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    if direction != Vec3::ZERO {
        direction = direction.normalize();
        state.target_look_at += direction * settings.pan_speed * time.delta_secs();
    }
}

/// Handle mouse wheel input to zoom camera
fn handle_camera_zoom_input(
    mut scroll_events: MessageReader<MouseWheel>,
    settings: Res<CameraSettings>,
    mut state: ResMut<CameraState>,
) {
    for event in scroll_events.read() {
        state.target_distance -= event.y * settings.zoom_speed;
        state.target_distance = state.target_distance.clamp(settings.min_distance, settings.max_distance);
    }
}

/// Clamp camera target to the current floor's boundaries
fn clamp_camera_to_bounds(
    mut state: ResMut<CameraState>,
    build_mode: Res<State<BuildMode>>,
    build_state: Option<Res<BuildModeState>>,
) {
    // Determine the active floor index
    let floor_index = if *build_mode.get() != BuildMode::Disabled {
        build_state.map(|s| s.active_floor).unwrap_or(0)
    } else {
        // In combat mode, default to floor 0 for now
        // Could be extended to track combat floor
        0
    };

    // Get world bounds for this floor
    let (world_min, world_max) = world_bounds_for_floor(floor_index);

    // Clamp the target look-at position to within bounds
    // Use X for world X, Z for world Z (Y is vertical)
    state.target_look_at.x = state.target_look_at.x.clamp(world_min.x, world_max.x);
    state.target_look_at.z = state.target_look_at.z.clamp(world_min.y, world_max.y);
}

/// Smoothly interpolate camera position toward target
fn update_camera_smooth(
    settings: Res<CameraSettings>,
    mut state: ResMut<CameraState>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let t = (state.smoothing * dt).min(1.0);

    // Smooth ease using exponential decay (feels like ease-out)
    state.current_look_at = state.current_look_at.lerp(state.target_look_at, t);
    state.current_distance = state.current_distance + (state.target_distance - state.current_distance) * t;

    // Update camera transform
    let Ok(mut transform) = camera_query.single_mut() else {
        return;
    };

    let (position, rotation) = calculate_camera_transform(state.current_distance, &settings, state.current_look_at);
    transform.translation = position;
    transform.rotation = rotation;
}

/// Calculate camera position and rotation from settings
/// Returns (position, rotation) for the camera transform
fn calculate_camera_transform(distance: f32, settings: &CameraSettings, look_at: Vec3) -> (Vec3, Quat) {
    let pitch_rad = settings.pitch_degrees.to_radians();
    let yaw_rad = settings.yaw_degrees.to_radians();

    // Calculate offset from look_at point
    // At pitch=0 (horizontal), camera is at distance along XZ plane
    // At pitch=90 (straight down), camera is directly above
    let horizontal_distance = distance * pitch_rad.cos();
    let vertical_distance = distance * pitch_rad.sin();

    let offset = Vec3::new(
        horizontal_distance * yaw_rad.sin(),
        vertical_distance,
        horizontal_distance * yaw_rad.cos(),
    );

    let position = look_at + offset;

    // Create rotation to look at target
    let forward = (look_at - position).normalize();
    let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);

    (position, rotation)
}

/// Camera height above the game plane (approximate, for compatibility)
pub const CAMERA_HEIGHT: f32 = 400.0;

/// Viewport height in world units (approximate, perspective varies with distance)
pub const VIEWPORT_HEIGHT: f32 = 720.0;

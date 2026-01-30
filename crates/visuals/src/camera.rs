//! 3D perspective camera with angled isometric-style view

use bevy::prelude::*;

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
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            distance: 400.0,
            pitch_degrees: 55.0,  // Angled down to see 3D depth
            yaw_degrees: 0.0,    // Looking along -Z (north in game terms)
            fov_degrees: 50.0,
        }
    }
}

/// Plugin that sets up the 3D perspective camera
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraSettings::default())
            .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands, settings: Res<CameraSettings>) {
    let (position, rotation) = calculate_camera_transform(&settings, Vec3::ZERO);

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

/// Calculate camera position and rotation from settings
/// Returns (position, rotation) for the camera transform
fn calculate_camera_transform(settings: &CameraSettings, look_at: Vec3) -> (Vec3, Quat) {
    let pitch_rad = settings.pitch_degrees.to_radians();
    let yaw_rad = settings.yaw_degrees.to_radians();

    // Calculate offset from look_at point
    // At pitch=0 (horizontal), camera is at distance along XZ plane
    // At pitch=90 (straight down), camera is directly above
    let horizontal_distance = settings.distance * pitch_rad.cos();
    let vertical_distance = settings.distance * pitch_rad.sin();

    let offset = Vec3::new(
        horizontal_distance * yaw_rad.sin(),
        vertical_distance,
        horizontal_distance * yaw_rad.cos(),
    );

    let position = look_at + offset;

    // Create rotation to look at target
    // We use look_to which expects the direction to look (negated offset)
    let forward = (look_at - position).normalize();
    let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);

    (position, rotation)
}

/// Camera height above the game plane (approximate, for compatibility)
pub const CAMERA_HEIGHT: f32 = 400.0;

/// Viewport height in world units (approximate, perspective varies with distance)
pub const VIEWPORT_HEIGHT: f32 = 720.0;

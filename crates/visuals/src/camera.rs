//! 3D orthographic camera for top-down view

use bevy::{camera::ScalingMode, prelude::*};

/// Marker component for the main game camera
#[derive(Component)]
pub struct MainCamera;

/// Plugin that sets up the 3D orthographic camera
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    // Spawn orthographic 3D camera looking down at the grid
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            // Fixed vertical height ensures consistent view
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 720.0,
            },
            near: 0.1,
            far: 1000.0,
            ..OrthographicProjection::default_3d()
        }),
        // Position camera high above, looking down
        Transform::from_xyz(0.0, 500.0, 0.1)
            .looking_at(Vec3::ZERO, Vec3::Z),
    ));
}

/// Camera height above the game plane
pub const CAMERA_HEIGHT: f32 = 500.0;

/// Viewport height in world units
pub const VIEWPORT_HEIGHT: f32 = 720.0;

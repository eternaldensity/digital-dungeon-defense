//! Spheroid mesh generation for creature entities

use bevy::prelude::*;

/// Creates a unit sphere mesh that can be scaled to create elliptical spheroids
///
/// # Arguments
/// * `segments` - Horizontal subdivisions (more = smoother)
/// * `rings` - Vertical subdivisions (more = smoother)
///
/// To create an elliptical spheroid, apply non-uniform scale via Transform:
/// - `Transform::from_scale(Vec3::new(1.0, 0.5, 1.0))` for a flattened disc
pub fn create_spheroid_mesh(segments: u32, rings: u32) -> Mesh {
    // Use Bevy's built-in sphere and convert
    Sphere::new(1.0)
        .mesh()
        .uv(segments, rings)
}

/// Scale factors for creature spheroids (flattened on Y axis for top-down view)
pub const CREATURE_SCALE: Vec3 = Vec3::new(1.0, 0.4, 1.0);

/// Scale factors for small creatures
pub const SMALL_CREATURE_SCALE: Vec3 = Vec3::new(0.6, 0.3, 0.6);

/// Scale factors for large creatures
pub const LARGE_CREATURE_SCALE: Vec3 = Vec3::new(1.5, 0.5, 1.5);

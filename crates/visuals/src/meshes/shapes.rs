//! Shape mesh generation for various entity types

use bevy::prelude::*;

/// Creates a hexagonal cylinder mesh for spawner entities
pub fn create_spawner_mesh() -> Mesh {
    // Use a cylinder with 6 sides for hexagonal look
    Cylinder::new(1.0, 0.5)
        .mesh()
        .resolution(6)
        .build()
}

/// Creates a cuboid mesh for firewall barriers
pub fn create_firewall_mesh() -> Mesh {
    Cuboid::new(2.0, 1.0, 0.3).mesh().build()
}

/// Creates a cone mesh for DataSpike projectiles
pub fn create_data_spike_mesh() -> Mesh {
    // Cone pointing upward
    Cone::new(0.3, 0.8)
        .mesh()
        .resolution(8)
        .build()
}

/// Creates an icosphere mesh for the dungeon core
pub fn create_core_mesh() -> Mesh {
    Sphere::new(1.5)
        .mesh()
        .ico(2)
        .unwrap()
}

/// Creates a small sphere for projectiles
pub fn create_projectile_mesh() -> Mesh {
    Sphere::new(0.2)
        .mesh()
        .uv(8, 6)
}

/// Size constants for entities
pub mod sizes {
    /// Base size for FloofBytes
    pub const FLOOFBYTE: f32 = 16.0;
    /// Base size for viruses
    pub const VIRUS: f32 = 20.0;
    /// Base size for spawners
    pub const SPAWNER: f32 = 32.0;
    /// Base size for dungeon core
    pub const CORE: f32 = 48.0;
    /// Base size for firewall segments
    pub const FIREWALL: f32 = 32.0;
}

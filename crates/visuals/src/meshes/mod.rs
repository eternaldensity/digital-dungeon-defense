//! Pre-built meshes for Tron-style entities

pub mod spheroid;
mod shapes;

pub use spheroid::*;
pub use shapes::*;

use bevy::prelude::*;

/// Pre-created mesh handles for all entity types
#[derive(Resource)]
pub struct TronMeshes {
    /// Flattened sphere for creatures
    pub creature_spheroid: Handle<Mesh>,
    /// Cylinder for spawners
    pub spawner: Handle<Mesh>,
    /// Cuboid for firewall barriers
    pub firewall: Handle<Mesh>,
    /// Cone/pyramid for DataSpike
    pub data_spike: Handle<Mesh>,
    /// Icosphere for dungeon core
    pub core: Handle<Mesh>,
    /// Small sphere for projectiles
    pub projectile: Handle<Mesh>,
}

/// Plugin that creates mesh resources
pub struct MeshesPlugin;

impl Plugin for MeshesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_meshes);
    }
}

fn setup_meshes(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let tron_meshes = TronMeshes {
        creature_spheroid: meshes.add(create_spheroid_mesh(16, 12)),
        spawner: meshes.add(create_spawner_mesh()),
        firewall: meshes.add(create_firewall_mesh()),
        data_spike: meshes.add(create_data_spike_mesh()),
        core: meshes.add(create_core_mesh()),
        projectile: meshes.add(create_projectile_mesh()),
    };

    commands.insert_resource(tron_meshes);
}

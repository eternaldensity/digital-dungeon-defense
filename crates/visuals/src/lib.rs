//! 3D Tron Legacy visual system for Digital Dungeon Defense
//!
//! Transforms 2D sprites into stylized 3D meshes with:
//! - Black background with neon highlights
//! - Glassy, translucent materials
//! - Elliptical spheroid creatures
//! - Glowing grid floor
//! - Bloom post-processing

mod palette;
mod camera;
mod lighting;
mod bloom;
mod grid;
mod sync;
mod materials;
mod meshes;

pub use palette::*;
pub use camera::*;
pub use lighting::*;
pub use bloom::*;
pub use grid::*;
pub use sync::*;
pub use materials::*;
pub use meshes::*;

use bevy::prelude::*;

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Sub-plugins
            .add_plugins((
                CameraPlugin,
                LightingPlugin,
                BloomPlugin,
                GridPlugin,
                MaterialsPlugin,
                MeshesPlugin,
                SyncPlugin,
            ));
    }
}

use bevy::prelude::*;

use ddd_core::CorePlugin;
use ddd_creatures::CreaturesPlugin;
use ddd_abilities::AbilitiesPlugin;
use ddd_floors::FloorsPlugin;
use ddd_traps::TrapsPlugin;

mod demo;
use demo::DemoPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Digital Dungeon Defense".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        // Game plugins - each self-contained and extensible
        .add_plugins((
            CorePlugin,
            CreaturesPlugin,
            AbilitiesPlugin,
            FloorsPlugin,
            TrapsPlugin,
        ))
        // Demo scene
        .add_plugins(DemoPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 100.0),
    ));
}

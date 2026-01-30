use bevy::prelude::*;

use ddd_core::CorePlugin;
use ddd_creatures::CreaturesPlugin;
use ddd_abilities::AbilitiesPlugin;
use ddd_floors::FloorsPlugin;
use ddd_traps::TrapsPlugin;
use ddd_visuals::VisualsPlugin;

mod demo;
use demo::DemoPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Digital Dungeon Defense".into(),
                resolution: (1280u32, 720u32).into(),
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
            VisualsPlugin,
        ))
        // Demo scene
        .add_plugins(DemoPlugin)
        .run();
}

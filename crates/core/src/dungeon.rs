//! Dungeon grid and floor management

use bevy::prelude::*;

/// Grid-based dungeon representation
#[derive(Resource)]
pub struct DungeonGrid {
    pub floors: Vec<Floor>,
    pub active_floor_index: usize,
    pub tile_size: f32,
    pub floor_width: usize,
    pub floor_height: usize,
}

impl Default for DungeonGrid {
    fn default() -> Self {
        Self {
            floors: vec![Floor::new(0)],
            active_floor_index: 0,
            tile_size: 32.0,
            floor_width: 20,
            floor_height: 15,
        }
    }
}

impl DungeonGrid {
    /// Get the three active floors (current, above, below)
    pub fn active_floors(&self) -> Vec<usize> {
        let mut active = Vec::new();
        if self.active_floor_index > 0 {
            active.push(self.active_floor_index - 1);
        }
        active.push(self.active_floor_index);
        if self.active_floor_index + 1 < self.floors.len() {
            active.push(self.active_floor_index + 1);
        }
        active
    }

    pub fn world_to_grid(&self, world_pos: Vec2) -> Option<GridPosition> {
        let x = ((world_pos.x / self.tile_size) + (self.floor_width as f32 / 2.0)) as i32;
        let y = ((world_pos.y / self.tile_size) + (self.floor_height as f32 / 2.0)) as i32;

        if x >= 0 && x < self.floor_width as i32 && y >= 0 && y < self.floor_height as i32 {
            Some(GridPosition {
                x: x as usize,
                y: y as usize,
                floor: self.active_floor_index,
            })
        } else {
            None
        }
    }

    pub fn grid_to_world(&self, grid_pos: &GridPosition) -> Vec2 {
        Vec2::new(
            (grid_pos.x as f32 - self.floor_width as f32 / 2.0 + 0.5) * self.tile_size,
            (grid_pos.y as f32 - self.floor_height as f32 / 2.0 + 0.5) * self.tile_size,
        )
    }

    pub fn add_floor(&mut self) -> usize {
        let index = self.floors.len();
        self.floors.push(Floor::new(index));
        index
    }
}

/// A single dungeon floor
pub struct Floor {
    pub index: usize,
    pub tiles: Vec<Vec<TileType>>,
    pub floor_type: Option<Entity>, // Entity with floor type components
}

impl Floor {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            tiles: vec![vec![TileType::Empty; 20]; 15],
            floor_type: None,
        }
    }
}

/// Types of tiles in the dungeon
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum TileType {
    #[default]
    Empty,
    Wall,
    Trap,
    Spawner,
    Core,
    Path,
}

/// Grid position component
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GridPosition {
    pub x: usize,
    pub y: usize,
    pub floor: usize,
}

impl GridPosition {
    pub fn new(x: usize, y: usize, floor: usize) -> Self {
        Self { x, y, floor }
    }
}

/// Marker for the dungeon core entity
#[derive(Component)]
pub struct DungeonCore;

/// Which floor an entity is currently on
#[derive(Component)]
pub struct OnFloor(pub usize);

/// Team affiliation
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Reflect)]
#[reflect(Component)]
pub enum Team {
    Player,
    Enemy,
}

impl Default for Team {
    fn default() -> Self {
        Self::Player
    }
}

/// Sets up the initial dungeon
pub fn setup_dungeon(
    mut commands: Commands,
    dungeon: Res<DungeonGrid>,
) {
    // Spawn the dungeon core at center of floor 0
    let core_pos = GridPosition::new(
        dungeon.floor_width / 2,
        dungeon.floor_height / 2,
        0,
    );
    let world_pos = dungeon.grid_to_world(&core_pos);

    commands.spawn((
        DungeonCore,
        core_pos,
        OnFloor(0),
        Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
        Sprite {
            color: Color::srgb(0.2, 0.8, 1.0),
            custom_size: Some(Vec2::splat(48.0)),
            ..default()
        },
    ));
}

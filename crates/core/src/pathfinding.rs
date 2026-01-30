//! Pathfinding for creatures navigating the dungeon

use bevy::prelude::*;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

use crate::{DungeonGrid, GridPosition, TileType};

/// Component for entities that need pathfinding
#[derive(Component)]
pub struct PathAgent {
    pub path: Vec<GridPosition>,
    pub current_index: usize,
    pub target: Option<GridPosition>,
}

impl Default for PathAgent {
    fn default() -> Self {
        Self {
            path: Vec::new(),
            current_index: 0,
            target: None,
        }
    }
}

impl PathAgent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current_waypoint(&self) -> Option<&GridPosition> {
        self.path.get(self.current_index)
    }

    pub fn advance(&mut self) {
        if self.current_index < self.path.len() {
            self.current_index += 1;
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_index >= self.path.len()
    }

    pub fn set_path(&mut self, path: Vec<GridPosition>) {
        self.path = path;
        self.current_index = 0;
    }
}

/// A* pathfinding implementation
pub fn find_path(
    dungeon: &DungeonGrid,
    start: GridPosition,
    goal: GridPosition,
) -> Option<Vec<GridPosition>> {
    if start.floor != goal.floor {
        // Cross-floor pathfinding not yet implemented
        return None;
    }

    let floor = dungeon.floors.get(start.floor)?;

    #[derive(Clone, Eq, PartialEq)]
    struct Node {
        pos: GridPosition,
        cost: u32,
        priority: u32,
    }

    impl Ord for Node {
        fn cmp(&self, other: &Self) -> Ordering {
            other.priority.cmp(&self.priority) // Reverse for min-heap
        }
    }

    impl PartialOrd for Node {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut cost_so_far: HashMap<(usize, usize), u32> = HashMap::new();

    let start_key = (start.x, start.y);
    let goal_key = (goal.x, goal.y);

    open_set.push(Node {
        pos: start,
        cost: 0,
        priority: 0,
    });
    cost_so_far.insert(start_key, 0);

    let neighbors = [(0i32, 1i32), (0, -1), (1, 0), (-1, 0)];

    while let Some(current) = open_set.pop() {
        let current_key = (current.pos.x, current.pos.y);

        if current_key == goal_key {
            // Reconstruct path
            let mut path = vec![goal];
            let mut current = goal_key;
            while let Some(&prev) = came_from.get(&current) {
                path.push(GridPosition::new(prev.0, prev.1, start.floor));
                current = prev;
            }
            path.reverse();
            return Some(path);
        }

        for (dx, dy) in neighbors {
            let nx = current.pos.x as i32 + dx;
            let ny = current.pos.y as i32 + dy;

            if nx < 0 || ny < 0 {
                continue;
            }

            let nx = nx as usize;
            let ny = ny as usize;

            if nx >= dungeon.floor_width || ny >= dungeon.floor_height {
                continue;
            }

            // Check if tile is walkable
            if let Some(row) = floor.tiles.get(ny) {
                if let Some(tile) = row.get(nx) {
                    if *tile == TileType::Wall {
                        continue;
                    }
                }
            }

            let neighbor_key = (nx, ny);
            let new_cost = cost_so_far[&current_key] + 1;

            if !cost_so_far.contains_key(&neighbor_key) || new_cost < cost_so_far[&neighbor_key] {
                cost_so_far.insert(neighbor_key, new_cost);
                let heuristic = ((nx as i32 - goal.x as i32).abs()
                    + (ny as i32 - goal.y as i32).abs()) as u32;
                let priority = new_cost + heuristic;

                open_set.push(Node {
                    pos: GridPosition::new(nx, ny, start.floor),
                    cost: new_cost,
                    priority,
                });
                came_from.insert(neighbor_key, current_key);
            }
        }
    }

    None
}

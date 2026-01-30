//! Connectivity validation - ensures paths exist between important locations.

use crate::layout::{DungeonLayout, FloorLayout, ValidationError};
use crate::tiles::{ConnectivityNode, DoorId, PlacedFeature, TileTemplate, TileTemplateId};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Validate connectivity on a single floor.
/// Checks that all connectors (up/down) and the core are reachable from each other.
pub fn validate_floor_connectivity(
    floor: &FloorLayout,
    templates: &HashMap<TileTemplateId, TileTemplate>,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Build inter-tile connectivity graph
    let graph = build_floor_graph(floor, templates);

    // Find all important nodes (connectors and core)
    let mut important_nodes: Vec<(IVec2, u8, &str)> = Vec::new();

    for (pos, tile) in &floor.tiles {
        let Some(template) = templates.get(&tile.template_id) else {
            continue;
        };

        for slot in &template.slots {
            match tile.placed_features.get(&slot.slot_id) {
                Some(PlacedFeature::Core) => {
                    important_nodes.push((*pos, slot.slot_id, "Core"));
                }
                Some(PlacedFeature::UpConnector) => {
                    important_nodes.push((*pos, slot.slot_id, "UpConnector"));
                }
                Some(PlacedFeature::DownConnector) => {
                    important_nodes.push((*pos, slot.slot_id, "DownConnector"));
                }
                _ => {}
            }
        }
    }

    // Check connectivity between all pairs of important nodes
    for i in 0..important_nodes.len() {
        for j in (i + 1)..important_nodes.len() {
            let (pos1, slot1, _name1) = important_nodes[i];
            let (pos2, slot2, _name2) = important_nodes[j];

            let node1 = FloorGraphNode::Slot(pos1, slot1);
            let node2 = FloorGraphNode::Slot(pos2, slot2);

            if !graph.has_path(node1, node2) {
                errors.push(ValidationError::NoPath {
                    from: pos1,
                    to: pos2,
                });
            }
        }
    }

    errors
}

/// Validate entire dungeon connectivity and requirements.
pub fn validate_dungeon(layout: &DungeonLayout) -> Vec<(usize, Vec<ValidationError>)> {
    let mut all_errors = Vec::new();

    for (floor_idx, floor) in layout.floors.iter().enumerate() {
        let mut floor_errors = Vec::new();

        // Basic requirements check
        if floor_idx == 0 {
            // Bottom floor needs either core or down connector
            if !floor.has_core() && !floor.has_down_connector() {
                floor_errors.push(ValidationError::MissingDownConnectorOrCore);
            }
        }

        if floor_idx > 0 {
            // Non-bottom floors need down connector
            if !floor.has_down_connector() {
                floor_errors.push(ValidationError::MissingDownConnector);
            }
        }

        if floor_idx < layout.floors.len() - 1 {
            // Non-top floors need up connector
            if !floor.has_up_connector() {
                floor_errors.push(ValidationError::MissingUpConnector);
            }
        }

        // Door matching
        floor_errors.extend(super::validate_door_matching(floor, &layout.templates));

        // Internal connectivity
        floor_errors.extend(validate_floor_connectivity(floor, &layout.templates));

        if !floor_errors.is_empty() {
            all_errors.push((floor_idx, floor_errors));
        }
    }

    // Check that dungeon has a core somewhere
    if !layout.has_core() {
        all_errors.push((0, vec![ValidationError::MissingCore]));
    }

    all_errors
}

/// A node in the floor connectivity graph.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FloorGraphNode {
    /// A door on a tile at (position, door_id).
    Door(IVec2, DoorId),
    /// A slot on a tile at (position, slot_id).
    Slot(IVec2, u8),
}

/// Floor connectivity graph for pathfinding between tiles.
pub struct FloorGraph {
    /// Adjacency list: node -> set of connected nodes.
    edges: HashMap<FloorGraphNode, HashSet<FloorGraphNode>>,
}

impl FloorGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    /// Add a bidirectional edge.
    pub fn add_edge(&mut self, a: FloorGraphNode, b: FloorGraphNode) {
        self.edges.entry(a).or_default().insert(b);
        self.edges.entry(b).or_default().insert(a);
    }

    /// Check if there's a path between two nodes using BFS.
    pub fn has_path(&self, start: FloorGraphNode, goal: FloorGraphNode) -> bool {
        if start == goal {
            return true;
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(start);
        queue.push_back(start);

        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = self.edges.get(&current) {
                for &neighbor in neighbors {
                    if neighbor == goal {
                        return true;
                    }
                    if visited.insert(neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        false
    }
}

impl Default for FloorGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a connectivity graph for a floor.
pub fn build_floor_graph(
    floor: &FloorLayout,
    templates: &HashMap<TileTemplateId, TileTemplate>,
) -> FloorGraph {
    let mut graph = FloorGraph::new();

    // For each tile, add internal connectivity
    for (pos, tile) in &floor.tiles {
        let Some(template) = templates.get(&tile.template_id) else {
            continue;
        };

        // Add edges within each connectivity component
        for component in &template.connectivity {
            let nodes: Vec<_> = component
                .iter()
                .map(|n| match n {
                    ConnectivityNode::Door(d) => {
                        FloorGraphNode::Door(*pos, tile.orientation.transform_door(*d))
                    }
                    ConnectivityNode::Slot(s) => FloorGraphNode::Slot(*pos, *s),
                })
                .collect();

            // Connect all nodes within the component
            for i in 0..nodes.len() {
                for j in (i + 1)..nodes.len() {
                    graph.add_edge(nodes[i], nodes[j]);
                }
            }
        }
    }

    // Add edges between adjacent tiles through matching doors
    for (pos, tile) in &floor.tiles {
        let Some(template) = templates.get(&tile.template_id) else {
            continue;
        };

        for door in &template.doors {
            let transformed_door = tile.orientation.transform_door(*door);
            let neighbor_pos = *pos + transformed_door.side.direction();

            if let Some(neighbor_tile) = floor.tiles.get(&neighbor_pos) {
                let Some(neighbor_template) = templates.get(&neighbor_tile.template_id) else {
                    continue;
                };

                // Find matching door on neighbor
                let required_door = transformed_door.matching_door();
                let neighbor_has_match = neighbor_template.doors.iter().any(|d| {
                    neighbor_tile.orientation.transform_door(*d) == required_door
                });

                if neighbor_has_match {
                    // Connect the two doors
                    let node1 = FloorGraphNode::Door(*pos, transformed_door);
                    let node2 = FloorGraphNode::Door(neighbor_pos, required_door);
                    graph.add_edge(node1, node2);
                }
            }
        }
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiles::{create_default_templates, PlacedTile};

    #[test]
    fn test_floor_graph_path() {
        let mut graph = FloorGraph::new();
        let a = FloorGraphNode::Slot(IVec2::ZERO, 0);
        let b = FloorGraphNode::Slot(IVec2::ZERO, 1);
        let c = FloorGraphNode::Slot(IVec2::ONE, 0);

        graph.add_edge(a, b);
        graph.add_edge(b, c);

        assert!(graph.has_path(a, c));
        assert!(graph.has_path(c, a));
    }

    #[test]
    fn test_floor_graph_no_path() {
        let graph = FloorGraph::new();
        let a = FloorGraphNode::Slot(IVec2::ZERO, 0);
        let b = FloorGraphNode::Slot(IVec2::ONE, 0);

        // No edges added
        assert!(!graph.has_path(a, b));
    }

    #[test]
    fn test_build_floor_graph_single_tile() {
        let mut floor = FloorLayout::new(0);
        let templates = create_default_templates();

        // Place a basic room
        floor.place_tile(IVec2::ZERO, PlacedTile::new(TileTemplateId(1), IVec2::ZERO));

        let graph = build_floor_graph(&floor, &templates);

        // All doors and slots in basic room should be connected
        // (it has one connectivity component)
        let template = templates.get(&TileTemplateId(1)).unwrap();
        let door1 = template.doors.iter().next().unwrap();
        let slot0 = template.slots.first().unwrap();

        let node1 = FloorGraphNode::Door(IVec2::ZERO, *door1);
        let node2 = FloorGraphNode::Slot(IVec2::ZERO, slot0.slot_id);

        assert!(graph.has_path(node1, node2));
    }

    #[test]
    fn test_validate_dungeon_missing_core() {
        let layout = DungeonLayout::new(); // Empty dungeon
        let errors = validate_dungeon(&layout);

        // Should have MissingCore error
        assert!(errors.iter().any(|(_, errs)| errs
            .iter()
            .any(|e| matches!(e, ValidationError::MissingCore))));
    }
}

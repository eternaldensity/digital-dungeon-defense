//! Door matching validation - ensures adjacent tiles have compatible doors.

use crate::layout::{FloorLayout, ValidationError};
use crate::tiles::{DoorId, TileSide, TileTemplate, TileTemplateId};
use bevy::prelude::*;
use std::collections::HashMap;

/// Validate that all doors on a floor have matching adjacent doors.
pub fn validate_door_matching(
    floor: &FloorLayout,
    templates: &HashMap<TileTemplateId, TileTemplate>,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for (pos, tile) in &floor.tiles {
        let Some(template) = templates.get(&tile.template_id) else {
            continue;
        };

        // Check each door on this tile
        for door in &template.doors {
            // Transform door according to tile orientation
            let transformed_door = tile.orientation.transform_door(*door);

            // Get the adjacent position
            let neighbor_pos = *pos + transformed_door.side.direction();

            // Check if there's a tile at the neighbor position
            if let Some(neighbor_tile) = floor.tiles.get(&neighbor_pos) {
                // Get neighbor template
                let Some(neighbor_template) = templates.get(&neighbor_tile.template_id) else {
                    continue;
                };

                // The neighbor needs a door on the opposite side, at the mirrored position
                let required_door = transformed_door.matching_door();

                // Check if neighbor has this door (after orientation transform)
                let neighbor_has_matching = neighbor_template.doors.iter().any(|d| {
                    neighbor_tile.orientation.transform_door(*d) == required_door
                });

                if !neighbor_has_matching {
                    errors.push(ValidationError::MismatchedDoors {
                        tile1: *pos,
                        tile2: neighbor_pos,
                        message: format!(
                            "Door at {:?} on tile {:?} has no matching door on adjacent tile",
                            transformed_door.side, pos
                        ),
                    });
                }
            } else {
                // Door leads to void - this might be intentional (edge of dungeon)
                // but we should flag it for awareness
                errors.push(ValidationError::DoorToVoid {
                    tile: *pos,
                    message: format!(
                        "Door at {:?} {:?} leads to empty space",
                        transformed_door.side, transformed_door.position
                    ),
                });
            }
        }
    }

    errors
}

/// Check if a tile can be placed at a position (doors would match neighbors).
pub fn can_place_tile(
    floor: &FloorLayout,
    templates: &HashMap<TileTemplateId, TileTemplate>,
    position: IVec2,
    template_id: TileTemplateId,
    orientation: crate::tiles::TileOrientation,
) -> bool {
    // Can't place if there's already a tile
    if floor.has_tile(position) {
        return false;
    }

    let Some(template) = templates.get(&template_id) else {
        return false;
    };

    // Check each side for compatibility with neighbors
    for side in TileSide::all() {
        let neighbor_pos = position + side.direction();

        if let Some(neighbor_tile) = floor.tiles.get(&neighbor_pos) {
            let Some(neighbor_template) = templates.get(&neighbor_tile.template_id) else {
                continue;
            };

            // Get doors on this side of the new tile
            let new_tile_doors: Vec<DoorId> = template
                .doors
                .iter()
                .map(|d| orientation.transform_door(*d))
                .filter(|d| d.side == side)
                .collect();

            // Get doors on the opposite side of the neighbor
            let neighbor_doors: Vec<DoorId> = neighbor_template
                .doors
                .iter()
                .map(|d| neighbor_tile.orientation.transform_door(*d))
                .filter(|d| d.side == side.opposite())
                .collect();

            // Each door on new tile needs a matching door on neighbor
            for door in &new_tile_doors {
                let required = door.matching_door();
                if !neighbor_doors.contains(&required) {
                    return false;
                }
            }

            // Each door on neighbor needs a matching door on new tile
            for door in &neighbor_doors {
                let required = door.matching_door();
                if !new_tile_doors.contains(&required) {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiles::{create_default_templates, PlacedTile, TileOrientation};

    #[test]
    fn test_empty_floor_no_errors() {
        let floor = FloorLayout::new(0);
        let templates = create_default_templates();
        let errors = validate_door_matching(&floor, &templates);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_single_tile_has_void_doors() {
        let mut floor = FloorLayout::new(0);
        let templates = create_default_templates();

        // Place a basic room (has 4 center doors)
        floor.place_tile(IVec2::ZERO, PlacedTile::new(TileTemplateId(1), IVec2::ZERO));

        let errors = validate_door_matching(&floor, &templates);
        // Should have 4 DoorToVoid errors (one per door)
        assert_eq!(
            errors
                .iter()
                .filter(|e| matches!(e, ValidationError::DoorToVoid { .. }))
                .count(),
            4
        );
    }

    #[test]
    fn test_can_place_adjacent_matching_tiles() {
        let mut floor = FloorLayout::new(0);
        let templates = create_default_templates();

        // Place a basic room at origin
        floor.place_tile(IVec2::ZERO, PlacedTile::new(TileTemplateId(1), IVec2::ZERO));

        // Should be able to place another basic room to the east (both have center doors)
        assert!(can_place_tile(
            &floor,
            &templates,
            IVec2::new(1, 0),
            TileTemplateId(1),
            TileOrientation::default()
        ));
    }

    #[test]
    fn test_cannot_place_mismatched_doors() {
        let mut floor = FloorLayout::new(0);
        let templates = create_default_templates();

        // Place a corridor (only E/W doors)
        floor.place_tile(IVec2::ZERO, PlacedTile::new(TileTemplateId(2), IVec2::ZERO));

        // Cannot place basic room to the north (corridor has no north door)
        assert!(!can_place_tile(
            &floor,
            &templates,
            IVec2::new(0, 1),
            TileTemplateId(1),
            TileOrientation::default()
        ));
    }
}

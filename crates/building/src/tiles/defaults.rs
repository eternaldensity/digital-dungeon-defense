//! Default tile templates.

use super::{
    ConnectivityNode, ConstructionSlot, DoorId, DoorPosition, SlotType, TileSide, TileTemplate,
    TileTemplateId, PLANNING_TILE_SIZE,
};
use std::collections::{HashMap, HashSet};

/// Create all default tile templates.
pub fn create_default_templates() -> HashMap<TileTemplateId, TileTemplate> {
    let mut templates = HashMap::new();

    templates.insert(TileTemplateId(1), create_basic_room());
    templates.insert(TileTemplateId(2), create_corridor());
    templates.insert(TileTemplateId(3), create_core_chamber());
    templates.insert(TileTemplateId(4), create_stairwell());
    templates.insert(TileTemplateId(5), create_l_room());
    templates.insert(TileTemplateId(6), create_t_junction());
    templates.insert(TileTemplateId(7), create_corner());

    templates
}

/// Basic Room - open room with all 4 center doors.
fn create_basic_room() -> TileTemplate {
    let doors: HashSet<DoorId> = [
        DoorId::new(TileSide::North, DoorPosition::Center),
        DoorId::new(TileSide::East, DoorPosition::Center),
        DoorId::new(TileSide::South, DoorPosition::Center),
        DoorId::new(TileSide::West, DoorPosition::Center),
    ]
    .into_iter()
    .collect();

    let slots = vec![
        ConstructionSlot::new(0.5, 0.5, SlotType::Trap, 0),
        ConstructionSlot::new(0.25, 0.25, SlotType::Spawner, 1),
    ];

    // All doors and slots connected in one component
    let connectivity = vec![{
        let mut set = HashSet::new();
        for door in &doors {
            set.insert(ConnectivityNode::Door(*door));
        }
        for slot in &slots {
            set.insert(ConnectivityNode::Slot(slot.slot_id));
        }
        set
    }];

    TileTemplate {
        id: TileTemplateId(1),
        name: "Basic Room".to_string(),
        doors,
        slots,
        connectivity,
        walkable_pattern: create_open_room_pattern(),
        visual_id: "basic_room".to_string(),
    }
}

/// Corridor - straight hallway with East/West center doors.
fn create_corridor() -> TileTemplate {
    let doors: HashSet<DoorId> = [
        DoorId::new(TileSide::East, DoorPosition::Center),
        DoorId::new(TileSide::West, DoorPosition::Center),
    ]
    .into_iter()
    .collect();

    let slots = vec![ConstructionSlot::new(0.5, 0.5, SlotType::Trap, 0)];

    let connectivity = vec![{
        let mut set = HashSet::new();
        for door in &doors {
            set.insert(ConnectivityNode::Door(*door));
        }
        set.insert(ConnectivityNode::Slot(0));
        set
    }];

    TileTemplate {
        id: TileTemplateId(2),
        name: "Corridor".to_string(),
        doors,
        slots,
        connectivity,
        walkable_pattern: create_corridor_pattern(),
        visual_id: "corridor".to_string(),
    }
}

/// Core Chamber - room for the dungeon core.
fn create_core_chamber() -> TileTemplate {
    let doors: HashSet<DoorId> = [
        DoorId::new(TileSide::North, DoorPosition::Center),
        DoorId::new(TileSide::East, DoorPosition::Center),
        DoorId::new(TileSide::South, DoorPosition::Center),
        DoorId::new(TileSide::West, DoorPosition::Center),
    ]
    .into_iter()
    .collect();

    let slots = vec![
        ConstructionSlot::new(0.5, 0.5, SlotType::Core, 0),
        ConstructionSlot::new(0.2, 0.2, SlotType::Spawner, 1),
        ConstructionSlot::new(0.8, 0.2, SlotType::Spawner, 2),
    ];

    let connectivity = vec![{
        let mut set = HashSet::new();
        for door in &doors {
            set.insert(ConnectivityNode::Door(*door));
        }
        for slot in &slots {
            set.insert(ConnectivityNode::Slot(slot.slot_id));
        }
        set
    }];

    TileTemplate {
        id: TileTemplateId(3),
        name: "Core Chamber".to_string(),
        doors,
        slots,
        connectivity,
        walkable_pattern: create_open_room_pattern(),
        visual_id: "core_chamber".to_string(),
    }
}

/// Stairwell - vertical floor connection with South door.
fn create_stairwell() -> TileTemplate {
    let doors: HashSet<DoorId> =
        [DoorId::new(TileSide::South, DoorPosition::Center)]
            .into_iter()
            .collect();

    let slots = vec![
        ConstructionSlot::new(0.5, 0.7, SlotType::UpConnector, 0),
        ConstructionSlot::new(0.5, 0.3, SlotType::DownConnector, 1),
    ];

    // All elements connected - you can go from door to either connector
    let connectivity = vec![{
        let mut set = HashSet::new();
        for door in &doors {
            set.insert(ConnectivityNode::Door(*door));
        }
        for slot in &slots {
            set.insert(ConnectivityNode::Slot(slot.slot_id));
        }
        set
    }];

    TileTemplate {
        id: TileTemplateId(4),
        name: "Stairwell".to_string(),
        doors,
        slots,
        connectivity,
        walkable_pattern: create_stairwell_pattern(),
        visual_id: "stairwell".to_string(),
    }
}

/// L-Room - asymmetric room with North-left and East-right doors.
fn create_l_room() -> TileTemplate {
    let doors: HashSet<DoorId> = [
        DoorId::new(TileSide::North, DoorPosition::Left),
        DoorId::new(TileSide::East, DoorPosition::Right),
    ]
    .into_iter()
    .collect();

    let slots = vec![
        ConstructionSlot::new(0.25, 0.75, SlotType::Trap, 0),
        ConstructionSlot::new(0.75, 0.25, SlotType::Spawner, 1),
    ];

    let connectivity = vec![{
        let mut set = HashSet::new();
        for door in &doors {
            set.insert(ConnectivityNode::Door(*door));
        }
        for slot in &slots {
            set.insert(ConnectivityNode::Slot(slot.slot_id));
        }
        set
    }];

    TileTemplate {
        id: TileTemplateId(5),
        name: "L-Room".to_string(),
        doors,
        slots,
        connectivity,
        walkable_pattern: create_l_room_pattern(),
        visual_id: "l_room".to_string(),
    }
}

/// T-Junction - three-way intersection with North/East/West doors.
fn create_t_junction() -> TileTemplate {
    let doors: HashSet<DoorId> = [
        DoorId::new(TileSide::North, DoorPosition::Center),
        DoorId::new(TileSide::East, DoorPosition::Center),
        DoorId::new(TileSide::West, DoorPosition::Center),
    ]
    .into_iter()
    .collect();

    let slots = vec![ConstructionSlot::new(0.5, 0.5, SlotType::Trap, 0)];

    let connectivity = vec![{
        let mut set = HashSet::new();
        for door in &doors {
            set.insert(ConnectivityNode::Door(*door));
        }
        set.insert(ConnectivityNode::Slot(0));
        set
    }];

    TileTemplate {
        id: TileTemplateId(6),
        name: "T-Junction".to_string(),
        doors,
        slots,
        connectivity,
        walkable_pattern: create_t_junction_pattern(),
        visual_id: "t_junction".to_string(),
    }
}

/// Corner - two-way turn with North/East doors.
fn create_corner() -> TileTemplate {
    let doors: HashSet<DoorId> = [
        DoorId::new(TileSide::North, DoorPosition::Center),
        DoorId::new(TileSide::East, DoorPosition::Center),
    ]
    .into_iter()
    .collect();

    let slots = vec![ConstructionSlot::new(0.7, 0.7, SlotType::Spawner, 0)];

    let connectivity = vec![{
        let mut set = HashSet::new();
        for door in &doors {
            set.insert(ConnectivityNode::Door(*door));
        }
        set.insert(ConnectivityNode::Slot(0));
        set
    }];

    TileTemplate {
        id: TileTemplateId(7),
        name: "Corner".to_string(),
        doors,
        slots,
        connectivity,
        walkable_pattern: create_corner_pattern(),
        visual_id: "corner".to_string(),
    }
}

// Walkable pattern helpers
// true = walkable, false = wall
// Pattern is indexed as [y][x] where y=0 is south, y=8 is north

/// Open room - walls on edges, open interior, doors at center positions.
fn create_open_room_pattern() -> [[bool; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE] {
    let mut pattern = [[false; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE];

    // Fill interior as walkable
    for y in 1..8 {
        for x in 1..8 {
            pattern[y][x] = true;
        }
    }

    // Add door openings (center position = index 4)
    pattern[0][4] = true; // South door
    pattern[8][4] = true; // North door
    pattern[4][0] = true; // West door
    pattern[4][8] = true; // East door

    pattern
}

/// Corridor - narrow walkway East-West.
fn create_corridor_pattern() -> [[bool; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE] {
    let mut pattern = [[false; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE];

    // 3-wide corridor in the center (rows 3, 4, 5)
    for y in 3..6 {
        for x in 0..PLANNING_TILE_SIZE {
            pattern[y][x] = true;
        }
    }

    pattern
}

/// Stairwell - mostly open with entry from south.
fn create_stairwell_pattern() -> [[bool; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE] {
    let mut pattern = [[false; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE];

    // Central area walkable
    for y in 1..8 {
        for x in 2..7 {
            pattern[y][x] = true;
        }
    }

    // South door opening
    pattern[0][4] = true;

    pattern
}

/// L-shaped room - walkable in L pattern.
fn create_l_room_pattern() -> [[bool; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE] {
    let mut pattern = [[false; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE];

    // Vertical leg (left side, going up)
    for y in 1..8 {
        for x in 1..4 {
            pattern[y][x] = true;
        }
    }

    // Horizontal leg (bottom, going right)
    for y in 1..4 {
        for x in 1..8 {
            pattern[y][x] = true;
        }
    }

    // North-left door (x=1, y=8)
    pattern[8][1] = true;

    // East-right door (x=8, y=7)
    pattern[7][8] = true;

    pattern
}

/// T-junction - three-way intersection.
fn create_t_junction_pattern() -> [[bool; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE] {
    let mut pattern = [[false; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE];

    // Horizontal corridor
    for y in 3..6 {
        for x in 0..PLANNING_TILE_SIZE {
            pattern[y][x] = true;
        }
    }

    // Vertical extension north
    for y in 5..PLANNING_TILE_SIZE {
        for x in 3..6 {
            pattern[y][x] = true;
        }
    }

    pattern
}

/// Corner - L-shaped turn.
fn create_corner_pattern() -> [[bool; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE] {
    let mut pattern = [[false; PLANNING_TILE_SIZE]; PLANNING_TILE_SIZE];

    // Vertical section (going north)
    for y in 3..PLANNING_TILE_SIZE {
        for x in 3..6 {
            pattern[y][x] = true;
        }
    }

    // Horizontal section (going east)
    for y in 3..6 {
        for x in 3..PLANNING_TILE_SIZE {
            pattern[y][x] = true;
        }
    }

    pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_templates() {
        let templates = create_default_templates();
        assert_eq!(templates.len(), 7);
        assert!(templates.contains_key(&TileTemplateId(1)));
        assert!(templates.contains_key(&TileTemplateId(7)));
    }

    #[test]
    fn test_basic_room_has_all_center_doors() {
        let room = create_basic_room();
        assert!(room.has_door(DoorId::new(TileSide::North, DoorPosition::Center)));
        assert!(room.has_door(DoorId::new(TileSide::East, DoorPosition::Center)));
        assert!(room.has_door(DoorId::new(TileSide::South, DoorPosition::Center)));
        assert!(room.has_door(DoorId::new(TileSide::West, DoorPosition::Center)));
    }

    #[test]
    fn test_core_chamber_has_core_slot() {
        let chamber = create_core_chamber();
        assert!(chamber.slots.iter().any(|s| s.slot_type == SlotType::Core));
    }

    #[test]
    fn test_stairwell_has_connectors() {
        let stairwell = create_stairwell();
        assert!(stairwell
            .slots
            .iter()
            .any(|s| s.slot_type == SlotType::UpConnector));
        assert!(stairwell
            .slots
            .iter()
            .any(|s| s.slot_type == SlotType::DownConnector));
    }

    #[test]
    fn test_open_room_pattern_walkable() {
        let pattern = create_open_room_pattern();
        // Center should be walkable
        assert!(pattern[4][4]);
        // Corners should be walls
        assert!(!pattern[0][0]);
        assert!(!pattern[8][8]);
        // Center doors should be open
        assert!(pattern[0][4]); // South
        assert!(pattern[8][4]); // North
        assert!(pattern[4][0]); // West
        assert!(pattern[4][8]); // East
    }
}

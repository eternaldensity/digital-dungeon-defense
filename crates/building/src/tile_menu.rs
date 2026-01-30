//! Tile selection menu overlay.
//!
//! Provides a modal menu for selecting tile templates with keyboard navigation:
//! - T: Toggle menu open/close
//! - Arrow keys / WASD: Navigate selection
//! - Enter / Space: Confirm selection
//! - Escape: Cancel and close menu

use crate::{BuildMode, BuildModeState, DungeonLayout, TileOrientation, TileTemplateId};
use bevy::prelude::*;

/// Plugin for the tile selection menu.
pub struct TileMenuPlugin;

impl Plugin for TileMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileMenuState>()
            .add_systems(
                Update,
                (
                    toggle_tile_menu,
                    handle_menu_navigation,
                    handle_menu_selection,
                    sync_menu_ui,
                )
                    .chain()
                    .run_if(resource_exists::<DungeonLayout>),
            )
            .add_systems(OnEnter(BuildMode::TileSelect), spawn_menu_ui)
            .add_systems(OnExit(BuildMode::TileSelect), despawn_menu_ui);
    }
}

/// State for the tile selection menu.
#[derive(Resource, Default)]
pub struct TileMenuState {
    /// Currently highlighted index in the menu.
    pub selected_index: usize,
    /// List of available template IDs in display order.
    pub template_ids: Vec<TileTemplateId>,
}

impl TileMenuState {
    /// Move selection up.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down.
    pub fn move_down(&mut self) {
        if !self.template_ids.is_empty() && self.selected_index < self.template_ids.len() - 1 {
            self.selected_index += 1;
        }
    }

    /// Get the currently selected template ID.
    pub fn selected_template(&self) -> Option<TileTemplateId> {
        self.template_ids.get(self.selected_index).copied()
    }

    /// Populate template list from the layout.
    pub fn populate_from_layout(&mut self, layout: &DungeonLayout) {
        let mut ids: Vec<_> = layout.templates.keys().copied().collect();
        ids.sort_by_key(|id| id.0);
        self.template_ids = ids;
        // Clamp selection to valid range
        if !self.template_ids.is_empty() && self.selected_index >= self.template_ids.len() {
            self.selected_index = self.template_ids.len() - 1;
        }
    }
}

/// Marker for menu UI root.
#[derive(Component)]
pub struct TileMenuUI;

/// Marker for individual menu item.
#[derive(Component)]
pub struct TileMenuItem {
    pub index: usize,
}

/// Marker for the selection highlight.
#[derive(Component)]
pub struct MenuSelectionHighlight;

/// Toggle tile menu with T key when in build mode.
fn toggle_tile_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<BuildMode>>,
    mut next_state: ResMut<NextState<BuildMode>>,
    mut menu_state: ResMut<TileMenuState>,
    layout: Res<DungeonLayout>,
) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        match *current_state.get() {
            BuildMode::View | BuildMode::TilePlace => {
                // Open menu
                menu_state.populate_from_layout(&layout);
                next_state.set(BuildMode::TileSelect);
            }
            BuildMode::TileSelect => {
                // Close menu without selecting
                next_state.set(BuildMode::View);
            }
            _ => {}
        }
    }
}

/// Handle arrow key / WASD navigation in the menu.
fn handle_menu_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<BuildMode>>,
    mut menu_state: ResMut<TileMenuState>,
) {
    if *current_state.get() != BuildMode::TileSelect {
        return;
    }

    // Up navigation
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        menu_state.move_up();
    }

    // Down navigation
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        menu_state.move_down();
    }
}

/// Handle menu selection with Enter/Space or cancel with Escape.
fn handle_menu_selection(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<BuildMode>>,
    mut next_state: ResMut<NextState<BuildMode>>,
    menu_state: Res<TileMenuState>,
    mut build_state: ResMut<BuildModeState>,
) {
    if *current_state.get() != BuildMode::TileSelect {
        return;
    }

    // Confirm selection
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) {
        if let Some(template_id) = menu_state.selected_template() {
            build_state.selected_template = Some(template_id);
            build_state.current_orientation = TileOrientation::default();
            next_state.set(BuildMode::TilePlace);
        }
    }

    // Cancel
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(BuildMode::View);
    }
}

/// Spawn the menu UI when entering TileSelect mode.
fn spawn_menu_ui(
    mut commands: Commands,
    layout: Res<DungeonLayout>,
    menu_state: Res<TileMenuState>,
) {
    // Root container - centered overlay
    commands
        .spawn((
            TileMenuUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // Menu panel
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        min_width: Val::Px(300.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.02, 0.02, 0.05, 0.95)),
                    BorderColor::all(Color::srgb(0.2, 0.8, 1.0)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("SELECT TILE"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.2, 0.8, 1.0)),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // Menu items
                    for (index, template_id) in menu_state.template_ids.iter().enumerate() {
                        let template = layout.get_template(*template_id);
                        let name = template
                            .map(|t| t.name.as_str())
                            .unwrap_or("Unknown");

                        let is_selected = index == menu_state.selected_index;
                        let (bg_color, text_color) = if is_selected {
                            (
                                Color::srgba(0.2, 0.8, 1.0, 0.3),
                                Color::srgb(1.0, 1.0, 1.0),
                            )
                        } else {
                            (
                                Color::srgba(0.0, 0.0, 0.0, 0.0),
                                Color::srgb(0.6, 0.6, 0.6),
                            )
                        };

                        panel
                            .spawn((
                                TileMenuItem { index },
                                Node {
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                                    margin: UiRect::vertical(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(bg_color),
                            ))
                            .with_children(|item| {
                                // Tile name
                                item.spawn((
                                    Text::new(name),
                                    TextFont {
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(text_color),
                                ));
                            });
                    }

                    // Instructions
                    panel.spawn((
                        Text::new("\n[W/S or Arrows] Navigate\n[Enter/Space] Select\n[Esc] Cancel"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.4, 0.4)),
                        Node {
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        },
                    ));
                });
        });
}

/// Despawn menu UI when exiting TileSelect mode.
fn despawn_menu_ui(mut commands: Commands, query: Query<Entity, With<TileMenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_related::<Children>();
        commands.entity(entity).despawn();
    }
}

/// Sync menu item visuals with current selection state.
fn sync_menu_ui(
    current_state: Res<State<BuildMode>>,
    menu_state: Res<TileMenuState>,
    mut items: Query<(&TileMenuItem, &mut BackgroundColor, &Children)>,
    mut text_query: Query<&mut TextColor>,
) {
    if *current_state.get() != BuildMode::TileSelect {
        return;
    }

    if !menu_state.is_changed() {
        return;
    }

    for (item, mut bg, children) in items.iter_mut() {
        let is_selected = item.index == menu_state.selected_index;

        if is_selected {
            *bg = BackgroundColor(Color::srgba(0.2, 0.8, 1.0, 0.3));
        } else {
            *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0));
        }

        // Update text colors for children
        for child in children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                // Skip the key hint (first child, keep it gray)
                // The tile name is the second child
                if is_selected {
                    *text_color = TextColor(Color::srgb(1.0, 1.0, 1.0));
                } else {
                    *text_color = TextColor(Color::srgb(0.6, 0.6, 0.6));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_state_navigation() {
        let mut state = TileMenuState {
            selected_index: 1,
            template_ids: vec![
                TileTemplateId(1),
                TileTemplateId(2),
                TileTemplateId(3),
            ],
        };

        state.move_up();
        assert_eq!(state.selected_index, 0);

        // Can't go above 0
        state.move_up();
        assert_eq!(state.selected_index, 0);

        state.move_down();
        assert_eq!(state.selected_index, 1);

        state.move_down();
        assert_eq!(state.selected_index, 2);

        // Can't go below last item
        state.move_down();
        assert_eq!(state.selected_index, 2);
    }

    #[test]
    fn test_selected_template() {
        let state = TileMenuState {
            selected_index: 1,
            template_ids: vec![
                TileTemplateId(5),
                TileTemplateId(3),
                TileTemplateId(7),
            ],
        };

        assert_eq!(state.selected_template(), Some(TileTemplateId(3)));
    }

    #[test]
    fn test_empty_menu() {
        let state = TileMenuState::default();
        assert_eq!(state.selected_template(), None);
    }
}

use bevy::prelude::*;

use crate::player::DeathCounter;
use crate::states::{GameState, SettingsOrigin};

/// Tracks which pause menu button is currently selected (0-indexed).
#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedPauseItem(pub usize);

/// Component attached to each pause button to identify its action.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum PauseAction {
    Resume,
    Settings,
    ToMenu,
}

const PAUSE_ITEMS: usize = 3;
const COLOR_SELECTED: Color = Color::srgb(0.3, 0.3, 0.7);
const COLOR_NORMAL: Color = Color::srgb(0.15, 0.15, 0.15);

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedPauseItem>()
            .add_systems(Update, pause_toggle)
            .add_systems(OnEnter(GameState::Paused), spawn_pause_overlay)
            .add_systems(
                Update,
                (pause_navigation, pause_mouse, pause_highlight, pause_action)
                    .chain()
                    .run_if(in_state(GameState::Paused)),
            );
    }
}

fn pause_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    match state.get() {
        GameState::Playing => next_state.set(GameState::Paused),
        GameState::Paused => next_state.set(GameState::Playing),
        _ => {}
    }
}

fn spawn_pause_overlay(mut commands: Commands, mut selected: ResMut<SelectedPauseItem>) {
    selected.0 = 0;

    commands
        .spawn((
            // Full-screen semi-transparent overlay
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            // Use high z-index so overlay renders on top of game UI
            GlobalZIndex(100),
            DespawnOnExit::<GameState>(GameState::Paused),
        ))
        .with_children(|overlay| {
            // Center panel
            overlay
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("PAUSED"),
                        TextFont {
                            font_size: 40.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(40.0)),
                            ..default()
                        },
                    ));

                    // Buttons
                    let buttons = [
                        ("Resume", PauseAction::Resume),
                        ("Settings", PauseAction::Settings),
                        ("Main Menu", PauseAction::ToMenu),
                    ];

                    for (label, action) in buttons {
                        panel
                            .spawn((
                                Button,
                                action,
                                Node {
                                    width: Val::Px(300.0),
                                    height: Val::Px(50.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::vertical(Val::Px(5.0)),
                                    ..default()
                                },
                                BackgroundColor(COLOR_NORMAL),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new(label),
                                    TextFont {
                                        font_size: 24.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });
        });
}

fn pause_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedPauseItem>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        selected.0 = if selected.0 == 0 {
            PAUSE_ITEMS - 1
        } else {
            selected.0 - 1
        };
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        selected.0 = (selected.0 + 1) % PAUSE_ITEMS;
    }
}

fn pause_mouse(
    mut selected: ResMut<SelectedPauseItem>,
    buttons: Query<(&PauseAction, &Interaction), Changed<Interaction>>,
) {
    let actions_order = [PauseAction::Resume, PauseAction::Settings, PauseAction::ToMenu];

    for (action, interaction) in &buttons {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            let index = actions_order.iter().position(|a| a == action).unwrap_or(0);
            selected.0 = index;
        }
    }
}

fn pause_highlight(
    selected: Res<SelectedPauseItem>,
    mut buttons: Query<(&PauseAction, &mut BackgroundColor)>,
) {
    let actions_order = [PauseAction::Resume, PauseAction::Settings, PauseAction::ToMenu];

    for (action, mut bg) in &mut buttons {
        let index = actions_order.iter().position(|a| a == action).unwrap_or(0);
        if index == selected.0 {
            *bg = BackgroundColor(COLOR_SELECTED);
        } else {
            *bg = BackgroundColor(COLOR_NORMAL);
        }
    }
}

fn pause_action(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedPauseItem>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings_origin: ResMut<SettingsOrigin>,
    mut commands: Commands,
    buttons: Query<(&PauseAction, &Interaction)>,
) {
    let enter = keyboard.just_pressed(KeyCode::Enter);
    let clicked = buttons.iter().any(|(_, i)| *i == Interaction::Pressed);

    if !enter && !clicked {
        return;
    }

    match selected.0 {
        0 => {
            next_state.set(GameState::Playing);
        }
        1 => {
            *settings_origin = SettingsOrigin::Paused;
            next_state.set(GameState::Settings);
        }
        2 => {
            commands.remove_resource::<DeathCounter>();
            next_state.set(GameState::Menu);
        }
        _ => {}
    }
}

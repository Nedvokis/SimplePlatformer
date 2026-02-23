use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

use crate::states::{GameState, SettingsOrigin};

/// Tracks which menu button is currently selected (0-indexed).
#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedMenuItem(pub usize);

/// Component attached to each menu button to identify its action.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    StartGame,
    Settings,
    Exit,
}

const MENU_ITEMS: usize = 3;
const COLOR_SELECTED: Color = Color::srgb(0.3, 0.3, 0.7);
const COLOR_NORMAL: Color = Color::srgb(0.15, 0.15, 0.15);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedMenuItem>()
            .add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(
                Update,
                (menu_navigation, menu_highlight, menu_action)
                    .chain()
                    .run_if(in_state(GameState::Menu)),
            );
    }
}

fn setup_menu(mut commands: Commands, mut selected: ResMut<SelectedMenuItem>) {
    selected.0 = 0;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            DespawnOnExit::<GameState>(GameState::Menu),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SIMPLE PLATFORMER"),
                TextFont {
                    font_size: 60.0,
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
                ("Play", MenuAction::StartGame),
                ("Settings", MenuAction::Settings),
                ("Quit", MenuAction::Exit),
            ];

            for (label, action) in buttons {
                parent
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
}

fn menu_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedMenuItem>,
) {
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        selected.0 = if selected.0 == 0 {
            MENU_ITEMS - 1
        } else {
            selected.0 - 1
        };
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        selected.0 = (selected.0 + 1) % MENU_ITEMS;
    }
}

fn menu_highlight(
    selected: Res<SelectedMenuItem>,
    mut buttons: Query<(&MenuAction, &mut BackgroundColor)>,
) {
    let actions_order = [MenuAction::StartGame, MenuAction::Settings, MenuAction::Exit];

    for (action, mut bg) in &mut buttons {
        let index = actions_order.iter().position(|a| a == action).unwrap_or(0);
        if index == selected.0 {
            *bg = BackgroundColor(COLOR_SELECTED);
        } else {
            *bg = BackgroundColor(COLOR_NORMAL);
        }
    }
}

fn menu_action(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedMenuItem>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings_origin: ResMut<SettingsOrigin>,
    mut exit_events: MessageWriter<AppExit>,
) {
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    match selected.0 {
        0 => {
            next_state.set(GameState::Playing);
        }
        1 => {
            *settings_origin = SettingsOrigin::Menu;
            next_state.set(GameState::Settings);
        }
        2 => {
            exit_events.write(AppExit::default());
        }
        _ => {}
    }
}

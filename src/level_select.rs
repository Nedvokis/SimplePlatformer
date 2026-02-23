use bevy::prelude::*;

use crate::level::{CurrentLevel, LEVELS};
use crate::progress::PlayerProgress;
use crate::states::GameState;

#[derive(Resource, Default)]
pub struct SelectedLevelItem(pub usize);

const COLOR_SELECTED: Color = Color::srgb(0.3, 0.3, 0.7);
const COLOR_NORMAL: Color = Color::srgb(0.15, 0.15, 0.15);
const COLOR_LOCKED: Color = Color::srgb(0.08, 0.08, 0.08);
const COLOR_LOCKED_TEXT: Color = Color::srgb(0.4, 0.4, 0.4);

#[derive(Component)]
pub struct LevelSelectRow(pub usize);

pub struct LevelSelectPlugin;

impl Plugin for LevelSelectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedLevelItem>()
            .add_systems(OnEnter(GameState::LevelSelect), setup_level_select)
            .add_systems(
                Update,
                (level_select_navigation, level_select_mouse, level_select_highlight, level_select_action)
                    .chain()
                    .run_if(in_state(GameState::LevelSelect)),
            );
    }
}

fn total_items() -> usize {
    LEVELS.len() + 1 // levels + Back
}

fn setup_level_select(
    mut commands: Commands,
    mut selected: ResMut<SelectedLevelItem>,
    progress: Res<PlayerProgress>,
) {
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
            DespawnOnExit::<GameState>(GameState::LevelSelect),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SELECT LEVEL"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Level buttons
            for i in 0..LEVELS.len() {
                let unlocked = i <= progress.max_unlocked_level;
                let label = if unlocked {
                    format!("Level {}", i + 1)
                } else {
                    format!("Level {} - Locked", i + 1)
                };
                let bg_color = if unlocked { COLOR_NORMAL } else { COLOR_LOCKED };
                let text_color = if unlocked { Color::WHITE } else { COLOR_LOCKED_TEXT };

                parent
                    .spawn((
                        Button,
                        LevelSelectRow(i),
                        Node {
                            width: Val::Px(300.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::vertical(Val::Px(5.0)),
                            ..default()
                        },
                        BackgroundColor(bg_color),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(text_color),
                        ));
                    });
            }

            // Back button
            parent
                .spawn((
                    Button,
                    LevelSelectRow(LEVELS.len()),
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(COLOR_NORMAL),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Back"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn level_select_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedLevelItem>,
) {
    let total = total_items();
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        selected.0 = if selected.0 == 0 { total - 1 } else { selected.0 - 1 };
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        selected.0 = (selected.0 + 1) % total;
    }
}

fn level_select_mouse(
    mut selected: ResMut<SelectedLevelItem>,
    buttons: Query<(&LevelSelectRow, &Interaction), Changed<Interaction>>,
) {
    for (row, interaction) in &buttons {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            selected.0 = row.0;
        }
    }
}

fn level_select_highlight(
    selected: Res<SelectedLevelItem>,
    progress: Res<PlayerProgress>,
    mut rows: Query<(&LevelSelectRow, &mut BackgroundColor)>,
) {
    for (row, mut bg) in &mut rows {
        let is_selected = row.0 == selected.0;
        if row.0 < LEVELS.len() {
            // Level button
            let unlocked = row.0 <= progress.max_unlocked_level;
            if is_selected {
                *bg = BackgroundColor(COLOR_SELECTED);
            } else if unlocked {
                *bg = BackgroundColor(COLOR_NORMAL);
            } else {
                *bg = BackgroundColor(COLOR_LOCKED);
            }
        } else {
            // Back button
            if is_selected {
                *bg = BackgroundColor(COLOR_SELECTED);
            } else {
                *bg = BackgroundColor(COLOR_NORMAL);
            }
        }
    }
}

fn level_select_action(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedLevelItem>,
    progress: Res<PlayerProgress>,
    mut current_level: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<GameState>>,
    buttons: Query<(&LevelSelectRow, &Interaction)>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
        return;
    }

    let enter = keyboard.just_pressed(KeyCode::Enter);
    let clicked = buttons.iter().any(|(_, i)| *i == Interaction::Pressed);

    if !enter && !clicked {
        return;
    }

    if selected.0 < LEVELS.len() {
        // Level selected
        if selected.0 <= progress.max_unlocked_level {
            current_level.0 = selected.0;
            next_state.set(GameState::Playing);
        }
        // Do nothing if locked
    } else {
        // Back
        next_state.set(GameState::Menu);
    }
}

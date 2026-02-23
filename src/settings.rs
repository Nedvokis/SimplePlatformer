use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};

use crate::progress::{PlayerProgress, save_progress};
use crate::states::{GameState, SettingsOrigin};

const SETTINGS_ITEMS: usize = 7;
const COLOR_SELECTED: Color = Color::srgb(0.3, 0.3, 0.7);
const COLOR_NORMAL: Color = Color::srgb(0.15, 0.15, 0.15);

#[derive(Resource)]
pub struct GameSettings {
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub resolution: (u32, u32),
    pub fullscreen: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            music_volume: 0.7,
            sfx_volume: 0.7,
            resolution: (1280, 720),
            fullscreen: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct SettingsChanged(pub bool);

#[derive(Resource, Default)]
pub struct ConfirmingReset(pub bool);

#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedSettingsItem(pub usize);

/// Marker for each settings row, indexed 0..4.
#[derive(Component, Debug, Clone)]
pub struct SettingsRow(pub usize);

/// Marker for the confirmation overlay root entity.
#[derive(Component)]
struct ResetOverlay;

/// Which confirmation button is selected: 0 = Yes, 1 = No.
#[derive(Resource, Default)]
pub struct SelectedConfirmItem(pub usize);

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>()
            .init_resource::<SelectedSettingsItem>()
            .init_resource::<SettingsChanged>()
            .init_resource::<ConfirmingReset>()
            .init_resource::<SelectedConfirmItem>()
            .add_systems(OnEnter(GameState::Settings), setup_settings)
            .add_systems(
                Update,
                (
                    settings_navigation,
                    settings_mouse,
                    settings_adjust,
                    settings_highlight,
                    settings_update_text,
                )
                    .chain()
                    .run_if(in_state(GameState::Settings)),
            );
    }
}

fn volume_bar(volume: f32) -> String {
    let filled = (volume * 10.0).round() as usize;
    let empty = 10 - filled;
    let bar: String = "#".repeat(filled) + &"-".repeat(empty);
    format!("{} {}%", bar, (volume * 100.0).round() as u32)
}

fn row_text(index: usize, settings: &GameSettings) -> String {
    match index {
        0 => format!("Music: {}", volume_bar(settings.music_volume)),
        1 => format!("Sound: {}", volume_bar(settings.sfx_volume)),
        2 => format!("Resolution: {}x{}", settings.resolution.0, settings.resolution.1),
        3 => {
            let mode = if settings.fullscreen {
                "Fullscreen"
            } else {
                "Windowed"
            };
            format!("Window: {}", mode)
        }
        4 => "Save".to_string(),
        5 => "Reset Progress".to_string(),
        6 => "Back".to_string(),
        _ => String::new(),
    }
}

fn setup_settings(
    mut commands: Commands,
    mut selected: ResMut<SelectedSettingsItem>,
    settings: Res<GameSettings>,
    mut changed: ResMut<SettingsChanged>,
    mut confirming: ResMut<ConfirmingReset>,
) {
    selected.0 = 0;
    changed.0 = false;
    confirming.0 = false;

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
            DespawnOnExit::<GameState>(GameState::Settings),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SETTINGS"),
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

            // Rows
            for i in 0..SETTINGS_ITEMS {
                parent
                    .spawn((
                        Button,
                        SettingsRow(i),
                        Node {
                            width: Val::Px(400.0),
                            height: Val::Px(45.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::vertical(Val::Px(2.5)),
                            ..default()
                        },
                        BackgroundColor(COLOR_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(row_text(i, &settings)),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

fn settings_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedSettingsItem>,
    confirming: Res<ConfirmingReset>,
) {
    if confirming.0 {
        return;
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        selected.0 = if selected.0 == 0 {
            SETTINGS_ITEMS - 1
        } else {
            selected.0 - 1
        };
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        selected.0 = (selected.0 + 1) % SETTINGS_ITEMS;
    }
}

fn settings_mouse(
    mut selected: ResMut<SelectedSettingsItem>,
    confirming: Res<ConfirmingReset>,
    mut selected_confirm: ResMut<SelectedConfirmItem>,
    buttons: Query<(&SettingsRow, &Interaction), Changed<Interaction>>,
) {
    for (row, interaction) in &buttons {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            if confirming.0 && row.0 >= 100 {
                selected_confirm.0 = row.0 - 100;
            } else if !confirming.0 && row.0 < 100 {
                selected.0 = row.0;
            }
        }
    }
}

fn spawn_reset_overlay(commands: &mut Commands, selected_confirm: &mut ResMut<SelectedConfirmItem>) {
    selected_confirm.0 = 1; // Default to "No" for safety

    commands
        .spawn((
            ResetOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(200),
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Reset Progress?"),
                        TextFont { font_size: 32.0, ..default() },
                        TextColor(Color::WHITE),
                        Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
                    ));
                    panel.spawn((
                        Text::new("All level progress will be lost."),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.4, 0.4)),
                        Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
                    ));
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(20.0),
                            ..default()
                        })
                        .with_children(|row| {
                            for (i, label) in ["Yes", "No"].iter().enumerate() {
                                row.spawn((
                                    Button,
                                    SettingsRow(100 + i),
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(45.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(COLOR_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(*label),
                                        TextFont { font_size: 22.0, ..default() },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            }
                        });
                });
        });
}

#[allow(clippy::too_many_arguments)]
fn settings_adjust(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedSettingsItem>,
    mut settings: ResMut<GameSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    origin: Res<SettingsOrigin>,
    mut windows: Query<&mut Window>,
    mut changed: ResMut<SettingsChanged>,
    mut confirming: ResMut<ConfirmingReset>,
    mut progress: ResMut<PlayerProgress>,
    mut selected_confirm: ResMut<SelectedConfirmItem>,
    overlay_query: Query<Entity, With<ResetOverlay>>,
    buttons_interaction: Query<(&SettingsRow, &Interaction)>,
) {
    if confirming.0 {
        if keyboard.just_pressed(KeyCode::Escape) {
            for entity in &overlay_query {
                commands.entity(entity).despawn();
            }
            confirming.0 = false;
            return;
        }
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            selected_confirm.0 = 0;
        }
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            selected_confirm.0 = 1;
        }
        let enter = keyboard.just_pressed(KeyCode::Enter);
        let popup_clicked = buttons_interaction
            .iter()
            .any(|(row, i)| row.0 >= 100 && *i == Interaction::Pressed);
        if enter || popup_clicked {
            if selected_confirm.0 == 0 {
                progress.max_unlocked_level = 0;
                save_progress(&progress);
            }
            for entity in &overlay_query {
                commands.entity(entity).despawn();
            }
            confirming.0 = false;
        }
        return;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        go_back(&origin, &mut next_state);
        return;
    }

    let enter = keyboard.just_pressed(KeyCode::Enter);
    let row_clicked = buttons_interaction
        .iter()
        .find(|(row, i)| row.0 < 100 && **i == Interaction::Pressed)
        .map(|(row, _)| row.0);

    if enter || row_clicked.is_some() {
        let action_row = row_clicked.unwrap_or(selected.0);
        match action_row {
            4 => {
                changed.0 = false;
                return;
            }
            5 => {
                confirming.0 = true;
                spawn_reset_overlay(&mut commands, &mut selected_confirm);
                return;
            }
            6 => {
                go_back(&origin, &mut next_state);
                return;
            }
            _ => {}
        }
    }

    let left = keyboard.just_pressed(KeyCode::ArrowLeft);
    let right = keyboard.just_pressed(KeyCode::ArrowRight);

    if !left && !right {
        return;
    }

    match selected.0 {
        0 => {
            if right {
                settings.music_volume = (settings.music_volume + 0.1).min(1.0);
            } else {
                settings.music_volume = (settings.music_volume - 0.1).max(0.0);
            }
            changed.0 = true;
        }
        1 => {
            if right {
                settings.sfx_volume = (settings.sfx_volume + 0.1).min(1.0);
            } else {
                settings.sfx_volume = (settings.sfx_volume - 0.1).max(0.0);
            }
            changed.0 = true;
        }
        2 => {
            settings.resolution = if settings.resolution == (1280, 720) {
                (1920, 1080)
            } else {
                (1280, 720)
            };
            if let Ok(mut window) = windows.single_mut() {
                window.resolution.set(settings.resolution.0 as f32, settings.resolution.1 as f32);
            }
            changed.0 = true;
        }
        3 => {
            settings.fullscreen = !settings.fullscreen;
            if let Ok(mut window) = windows.single_mut() {
                window.mode = if settings.fullscreen {
                    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                } else {
                    WindowMode::Windowed
                };
            }
            changed.0 = true;
        }
        _ => {}
    }
}

fn go_back(origin: &SettingsOrigin, next_state: &mut ResMut<NextState<GameState>>) {
    match origin {
        SettingsOrigin::Menu => next_state.set(GameState::Menu),
        SettingsOrigin::Paused => next_state.set(GameState::Paused),
    }
}

fn settings_highlight(
    selected: Res<SelectedSettingsItem>,
    changed: Res<SettingsChanged>,
    confirming: Res<ConfirmingReset>,
    selected_confirm: Res<SelectedConfirmItem>,
    mut rows: Query<(&SettingsRow, &mut BackgroundColor)>,
) {
    for (row, mut bg) in &mut rows {
        let is_selected = row.0 == selected.0;

        if row.0 >= 100 {
            let confirm_index = row.0 - 100;
            if confirming.0 && confirm_index == selected_confirm.0 {
                if confirm_index == 0 {
                    *bg = BackgroundColor(Color::srgb(0.7, 0.15, 0.15));
                } else {
                    *bg = BackgroundColor(COLOR_SELECTED);
                }
            } else {
                *bg = BackgroundColor(COLOR_NORMAL);
            }
            continue;
        }

        if row.0 == 4 && changed.0 {
            if is_selected {
                *bg = BackgroundColor(Color::srgb(0.2, 0.8, 0.2));
            } else {
                *bg = BackgroundColor(Color::srgb(0.1, 0.4, 0.1));
            }
        } else if is_selected {
            *bg = BackgroundColor(COLOR_SELECTED);
        } else {
            *bg = BackgroundColor(COLOR_NORMAL);
        }
    }
}

fn settings_update_text(
    settings: Res<GameSettings>,
    rows: Query<(&SettingsRow, &Children)>,
    mut texts: Query<&mut Text>,
) {
    for (row, children) in &rows {
        if row.0 >= 100 {
            continue;
        }
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                **text = row_text(row.0, &settings);
            }
        }
    }
}

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

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>()
            .init_resource::<SelectedSettingsItem>()
            .init_resource::<SettingsChanged>()
            .init_resource::<ConfirmingReset>()
            .add_systems(OnEnter(GameState::Settings), setup_settings)
            .add_systems(
                Update,
                (
                    settings_navigation,
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

fn row_text(index: usize, settings: &GameSettings, confirming_reset: bool) -> String {
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
        5 => {
            if confirming_reset {
                "Are you sure? Yes / No".to_string()
            } else {
                "Reset Progress".to_string()
            }
        }
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
                            Text::new(row_text(i, &settings, false)),
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
) {
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

fn settings_adjust(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedSettingsItem>,
    mut settings: ResMut<GameSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    origin: Res<SettingsOrigin>,
    mut windows: Query<&mut Window>,
    mut changed: ResMut<SettingsChanged>,
    mut confirming: ResMut<ConfirmingReset>,
    mut progress: ResMut<PlayerProgress>,
) {
    // Escape: cancel confirmation or go back
    if keyboard.just_pressed(KeyCode::Escape) {
        if confirming.0 {
            confirming.0 = false;
        } else {
            go_back(&origin, &mut next_state);
        }
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        match selected.0 {
            4 => {
                // Save
                changed.0 = false;
                return;
            }
            5 => {
                if confirming.0 {
                    // Yes - confirm reset
                    progress.max_unlocked_level = 0;
                    save_progress(&progress);
                    confirming.0 = false;
                } else {
                    // Enter confirmation mode
                    confirming.0 = true;
                }
                return;
            }
            6 => {
                // Back
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

    if confirming.0 && selected.0 == 5 {
        if left {
            // Yes
            progress.max_unlocked_level = 0;
            save_progress(&progress);
            confirming.0 = false;
        } else if right {
            // No
            confirming.0 = false;
        }
        return;
    }

    match selected.0 {
        0 => {
            // Music volume
            if right {
                settings.music_volume = (settings.music_volume + 0.1).min(1.0);
            } else {
                settings.music_volume = (settings.music_volume - 0.1).max(0.0);
            }
            changed.0 = true;
        }
        1 => {
            // SFX volume
            if right {
                settings.sfx_volume = (settings.sfx_volume + 0.1).min(1.0);
            } else {
                settings.sfx_volume = (settings.sfx_volume - 0.1).max(0.0);
            }
            changed.0 = true;
        }
        2 => {
            // Resolution toggle
            settings.resolution = if settings.resolution == (1280, 720) {
                (1920, 1080)
            } else {
                (1280, 720)
            };
            if let Ok(mut window) = windows.single_mut() {
                window
                    .resolution
                    .set(settings.resolution.0 as f32, settings.resolution.1 as f32);
            }
            changed.0 = true;
        }
        3 => {
            // Fullscreen toggle
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
    mut rows: Query<(&SettingsRow, &mut BackgroundColor)>,
) {
    for (row, mut bg) in &mut rows {
        let is_selected = row.0 == selected.0;
        if confirming.0 && row.0 == 5 {
            *bg = BackgroundColor(Color::srgb(0.7, 0.15, 0.15));
        } else if row.0 == 4 && changed.0 {
            // Save row with unsaved changes
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
    confirming: Res<ConfirmingReset>,
    rows: Query<(&SettingsRow, &Children)>,
    mut texts: Query<&mut Text>,
) {
    for (row, children) in &rows {
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                **text = row_text(row.0, &settings, confirming.0);
            }
        }
    }
}

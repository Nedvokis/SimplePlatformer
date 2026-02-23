use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};

use crate::states::{GameState, SettingsOrigin};

const SETTINGS_ITEMS: usize = 5;
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
    let bar: String = "\u{2588}".repeat(filled) + &"\u{2591}".repeat(empty);
    format!("{} {}%", bar, (volume * 100.0).round() as u32)
}

fn row_text(index: usize, settings: &GameSettings) -> String {
    match index {
        0 => format!("Музыка: {}", volume_bar(settings.music_volume)),
        1 => format!("Звуки: {}", volume_bar(settings.sfx_volume)),
        2 => format!("Разрешение: {}x{}", settings.resolution.0, settings.resolution.1),
        3 => {
            let mode = if settings.fullscreen {
                "Полный экран"
            } else {
                "Оконный"
            };
            format!("Экран: {}", mode)
        }
        4 => "Назад".to_string(),
        _ => String::new(),
    }
}

fn setup_settings(
    mut commands: Commands,
    mut selected: ResMut<SelectedSettingsItem>,
    settings: Res<GameSettings>,
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
            DespawnOnExit::<GameState>(GameState::Settings),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("НАСТРОЙКИ"),
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
) {
    // Escape: always go back
    if keyboard.just_pressed(KeyCode::Escape) {
        go_back(&origin, &mut next_state);
        return;
    }

    // Enter on "Назад"
    if keyboard.just_pressed(KeyCode::Enter) && selected.0 == 4 {
        go_back(&origin, &mut next_state);
        return;
    }

    let left = keyboard.just_pressed(KeyCode::ArrowLeft);
    let right = keyboard.just_pressed(KeyCode::ArrowRight);

    if !left && !right {
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
        }
        1 => {
            // SFX volume
            if right {
                settings.sfx_volume = (settings.sfx_volume + 0.1).min(1.0);
            } else {
                settings.sfx_volume = (settings.sfx_volume - 0.1).max(0.0);
            }
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
    mut rows: Query<(&SettingsRow, &mut BackgroundColor)>,
) {
    for (row, mut bg) in &mut rows {
        if row.0 == selected.0 {
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
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                **text = row_text(row.0, &settings);
            }
        }
    }
}

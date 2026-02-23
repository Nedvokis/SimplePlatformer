mod camera;
mod level;
mod level_select;
mod menu;
mod pause;
mod physics;
mod player;
mod progress;
mod settings;
mod states;

use bevy::prelude::*;
use camera::CameraPlugin;
use level::LevelPlugin;
use level_select::LevelSelectPlugin;
use menu::MenuPlugin;
use pause::PausePlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use progress::ProgressPlugin;
use settings::SettingsPlugin;
use states::StatesPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SimplePlatformer".to_string(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(StatesPlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(LevelSelectPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ProgressPlugin)
        .add_plugins(MenuPlugin)
        .add_plugins(PausePlugin)
        .add_plugins(SettingsPlugin)
        .run();
}

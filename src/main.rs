mod menu;
mod settings;
mod states;

use bevy::prelude::*;
use menu::MenuPlugin;
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
        .add_plugins(MenuPlugin)
        .add_plugins(SettingsPlugin)
        .run();
}

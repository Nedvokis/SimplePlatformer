mod states;

use bevy::prelude::*;
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
        .run();
}

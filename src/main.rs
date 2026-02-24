mod camera;
mod level;
mod level_select;
mod logging;
mod menu;
mod pause;
mod physics;
mod player;
mod progress;
mod settings;
mod states;
mod victory;

use bevy::prelude::*;
use camera::CameraPlugin;
use level::LevelPlugin;
use level_select::LevelSelectPlugin;
use logging::{LogBuffer, LoggingPlugin};
use menu::MenuPlugin;
use pause::PausePlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use progress::ProgressPlugin;
use settings::SettingsPlugin;
use states::StatesPlugin;
use victory::VictoryPlugin;

fn main() {
    let ring_buffer = logging::setup_tracing();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "SimplePlatformer".to_string(),
                        resolution: (1280u32, 720u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .disable::<bevy::log::LogPlugin>(),
        )
        .insert_resource(LogBuffer(ring_buffer))
        .add_plugins(LoggingPlugin)
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
        .add_plugins(VictoryPlugin)
        .run();
}

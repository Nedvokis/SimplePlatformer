use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    LevelSelect,
    LevelTransition,
    Playing,
    Paused,
    Settings,
    Victory,
}

/// Tracks where Settings was opened from, to return correctly.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub enum SettingsOrigin {
    #[default]
    Menu,
    Paused,
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<SettingsOrigin>();
    }
}

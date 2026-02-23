use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct PlayerProgress {
    pub max_unlocked_level: usize,
}

impl Default for PlayerProgress {
    fn default() -> Self {
        Self { max_unlocked_level: 0 }
    }
}

pub struct ProgressPlugin;

impl Plugin for ProgressPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load_progress());
    }
}

fn save_path() -> PathBuf {
    let base = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("simple_platformer");
    std::fs::create_dir_all(&dir).ok();
    dir.join("save.json")
}

fn load_progress() -> PlayerProgress {
    let path = save_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_progress(progress: &PlayerProgress) {
    let path = save_path();
    if let Ok(json) = serde_json::to_string_pretty(progress) {
        std::fs::write(path, json).ok();
    }
}

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Resource, Serialize, Deserialize, Clone, Default)]
pub struct PlayerProgress {
    pub max_unlocked_level: usize,
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
    match std::fs::read_to_string(&path) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(p) => {
                info!("Progress loaded from {:?}", path);
                p
            }
            Err(e) => {
                error!("Failed to parse progress: {}", e);
                PlayerProgress::default()
            }
        },
        Err(_) => {
            info!("No save file found, starting fresh");
            PlayerProgress::default()
        }
    }
}

pub fn save_progress(progress: &PlayerProgress) {
    let path = save_path();
    match serde_json::to_string_pretty(progress) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                error!("Failed to save progress: {}", e);
            } else {
                info!("Progress saved (max_level: {})", progress.max_unlocked_level);
            }
        }
        Err(e) => error!("Failed to serialize progress: {}", e),
    }
}

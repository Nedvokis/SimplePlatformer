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

fn save_progress_to(progress: &PlayerProgress, path: &std::path::Path) {
    match serde_json::to_string_pretty(progress) {
        Ok(json) => {
            if let Err(e) = std::fs::write(path, json) {
                error!("Failed to save progress: {}", e);
            } else {
                info!("Progress saved (max_level: {})", progress.max_unlocked_level);
            }
        }
        Err(e) => error!("Failed to serialize progress: {}", e),
    }
}

fn load_progress_from(path: &std::path::Path) -> PlayerProgress {
    match std::fs::read_to_string(path) {
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

fn load_progress() -> PlayerProgress {
    load_progress_from(&save_path())
}

pub fn save_progress(progress: &PlayerProgress) {
    save_progress_to(progress, &save_path());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_save.json");

        let progress = PlayerProgress { max_unlocked_level: 3 };
        save_progress_to(&progress, &path);
        let loaded = load_progress_from(&path);
        assert_eq!(loaded.max_unlocked_level, 3);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let loaded = load_progress_from(&path);
        assert_eq!(loaded.max_unlocked_level, 0);
    }

    #[test]
    fn load_corrupted_json_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupted.json");

        std::fs::write(&path, "not valid json {{{{").unwrap();
        let loaded = load_progress_from(&path);
        assert_eq!(loaded.max_unlocked_level, 0);
    }
}

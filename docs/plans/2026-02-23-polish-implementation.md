# SimplePlatformer Polish & Features Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix 3 bugs (tofu text, jerky movement, settings save) and add 3 features (level select, save system, 3 new levels).

**Architecture:** Translate all UI to English. Add `Friction::ZERO` to player. Add `SettingsChanged` resource and "Save" button. New `GameState::LevelSelect` state with level select screen. New `src/progress.rs` module for save/load via JSON. Expand LEVELS array to 5.

**Tech Stack:** Rust, Bevy 0.18, avian2d 0.5, serde + serde_json (save files), dirs (platform paths)

**Design doc:** `docs/plans/2026-02-23-polish-design.md`

---

## Task 1: Translate all UI text to English

**Files:**
- Modify: `src/menu.rs`
- Modify: `src/pause.rs`
- Modify: `src/settings.rs`

**Step 1: Update menu.rs**

Change the button labels in `setup_menu`:
```rust
let buttons = [
    ("Play", MenuAction::StartGame),
    ("Settings", MenuAction::Settings),
    ("Quit", MenuAction::Exit),
];
```

**Step 2: Update pause.rs**

Change title and button labels in `spawn_pause_overlay`:
- Title: `"PAUSED"` (was "ПАУЗА")
- Buttons: `"Resume"`, `"Settings"`, `"Main Menu"` (were "Продолжить", "Настройки", "В меню")

**Step 3: Update settings.rs**

Change `row_text` function:
```rust
fn row_text(index: usize, settings: &GameSettings) -> String {
    match index {
        0 => format!("Music: {}", volume_bar(settings.music_volume)),
        1 => format!("Sound: {}", volume_bar(settings.sfx_volume)),
        2 => format!("Resolution: {}x{}", settings.resolution.0, settings.resolution.1),
        3 => {
            let mode = if settings.fullscreen { "Fullscreen" } else { "Windowed" };
            format!("Window: {}", mode)
        }
        4 => "Back".to_string(),
        _ => String::new(),
    }
}
```

Change title: `"SETTINGS"` (was "НАСТРОЙКИ")

**Step 4: Verify compilation and run**

Run: `cargo run`
Expected: all text displays correctly in English, no tofu squares.

**Step 5: Commit**

```bash
git add src/menu.rs src/pause.rs src/settings.rs
git commit -m "fix: translate all UI text to English"
```

---

## Task 2: Fix jerky movement on platforms

**Files:**
- Modify: `src/player.rs`

**Step 1: Add Friction::ZERO to player entity**

In `spawn_player`, add `Friction::ZERO` to the player's component tuple:

```rust
// In spawn_player, add to the spawn tuple after LockedAxes:
Friction::ZERO,
```

The `Friction` type comes from `avian2d::prelude::*` which is already imported.

**Step 2: Verify**

Run: `cargo run`
Expected: player slides smoothly on platforms, no stuttering.

**Step 3: Commit**

```bash
git add src/player.rs
git commit -m "fix: remove player friction to fix jerky movement on platforms"
```

---

## Task 3: Add Save button to settings

**Files:**
- Modify: `src/settings.rs`

**Step 1: Add SettingsChanged resource**

```rust
#[derive(Resource, Default)]
pub struct SettingsChanged(pub bool);
```

Register in plugin: `.init_resource::<SettingsChanged>()`

**Step 2: Update SETTINGS_ITEMS from 5 to 6**

```rust
const SETTINGS_ITEMS: usize = 6;
```

**Step 3: Update row_text for new indices**

Row 4 becomes "Save", row 5 becomes "Back":
```rust
4 => "Save".to_string(),
5 => "Back".to_string(),
```

**Step 4: Set SettingsChanged(true) when any value changes**

In `settings_adjust`, after any match arm that modifies a setting value (indices 0-3), add:
```rust
changed.0 = true;
```
Add `mut changed: ResMut<SettingsChanged>` parameter to `settings_adjust`.

**Step 5: Handle Save and Back button indices**

In `settings_adjust`:
- Enter on index 4 (Save): set `changed.0 = false` (confirmed save)
- Enter on index 5 (Back): call `go_back()`
- Esc: call `go_back()`

**Step 6: Special highlight for Save button**

In `settings_highlight`, the Save row (index 4) gets special treatment:
- If `SettingsChanged.0 == true` AND selected: bright green `Color::srgb(0.2, 0.8, 0.2)`
- If `SettingsChanged.0 == true` AND not selected: dim green `Color::srgb(0.1, 0.4, 0.1)`
- If `SettingsChanged.0 == false`: same as normal (dim gray)

Add `changed: Res<SettingsChanged>` parameter to `settings_highlight`.

**Step 7: Reset SettingsChanged on enter**

In `setup_settings`: `changed.0 = false;`
Add `mut changed: ResMut<SettingsChanged>` parameter.

**Step 8: Verify**

Run: `cargo run`
Expected: Settings has 6 rows. Save is dim. Change a value → Save turns green. Press Enter on Save → Save goes dim again.

**Step 9: Commit**

```bash
git add src/settings.rs
git commit -m "feat: add Save button to settings with change indicator"
```

---

## Task 4: Add dependencies for save system

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add serde_json and dirs**

```toml
serde_json = "1"
dirs = "6"
```

**Step 2: Verify**

Run: `cargo check`

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add serde_json and dirs dependencies"
```

---

## Task 5: Player progress save system

**Files:**
- Create: `src/progress.rs`
- Modify: `src/main.rs`

**Step 1: Create src/progress.rs**

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Resource, Serialize, Deserialize, Clone)]
pub struct PlayerProgress {
    pub max_unlocked_level: usize,
}

impl Default for PlayerProgress {
    fn default() -> Self {
        Self { max_unlocked_level: 0 } // Level 0 (first) always unlocked
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
```

**Step 2: Register in main.rs**

Add `mod progress;` and `.add_plugins(progress::ProgressPlugin)`.

**Step 3: Update check_exit in level.rs to save progress**

In `check_exit`, when a level is completed and `current_level.0` is incremented:
```rust
if current_level.0 > progress.max_unlocked_level {
    progress.max_unlocked_level = current_level.0;
    crate::progress::save_progress(&progress);
}
```

Add `mut progress: ResMut<PlayerProgress>` to `check_exit` parameters.
Add `use crate::progress::PlayerProgress;` to level.rs imports.

**Step 4: Verify**

Run: `cargo check`

**Step 5: Commit**

```bash
git add src/progress.rs src/main.rs src/level.rs
git commit -m "feat: add player progress save/load system"
```

---

## Task 6: Add LevelSelect state and screen

**Files:**
- Modify: `src/states.rs`
- Create: `src/level_select.rs`
- Modify: `src/menu.rs`
- Modify: `src/main.rs`

**Step 1: Add LevelSelect to GameState**

In `src/states.rs`:
```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    LevelSelect,
    Playing,
    Paused,
    Settings,
}
```

**Step 2: Update menu.rs — "Play" now goes to LevelSelect**

In `menu_action`, change `StartGame` action:
```rust
0 => {
    next_state.set(GameState::LevelSelect);
}
```

**Step 3: Create src/level_select.rs**

The level select screen:
- `LevelSelectPlugin` with `OnEnter(GameState::LevelSelect)` → `setup_level_select`
- `SelectedLevelItem` resource (usize, 0..TOTAL_LEVELS where last index = "Back")
- UI: Title "SELECT LEVEL", grid of level buttons, "Back" button
- Each level button shows: number if unlocked, "Locked" if locked (check `PlayerProgress.max_unlocked_level`)
- Navigation: ↑/↓ + Enter (single column for simplicity)
- Enter on unlocked level: set `CurrentLevel(index)`, `NextState(Playing)`
- Enter on locked level: do nothing
- Enter on "Back" or Esc: `NextState(Menu)`
- Locked levels: dimmer background `Color::srgb(0.08, 0.08, 0.08)`, text `Color::srgb(0.4, 0.4, 0.4)`
- Unlocked levels: normal background, normal highlight when selected
- All entities: `DespawnOnExit::<GameState>(GameState::LevelSelect)`

**Important:** Need to know total level count. Import `LEVELS` from `level.rs` (make it `pub`).

**Step 4: Register in main.rs**

Add `mod level_select;` and `.add_plugins(level_select::LevelSelectPlugin)`.

**Step 5: Make LEVELS pub in level.rs**

Change `const LEVELS` to `pub const LEVELS` in `src/level.rs`.

**Step 6: Verify**

Run: `cargo run`
Expected: Menu → "Play" → Level Select screen. Only level 1 available. Select it → gameplay starts. After completing → level 2 unlocks.

**Step 7: Commit**

```bash
git add src/states.rs src/level_select.rs src/menu.rs src/main.rs src/level.rs
git commit -m "feat: add level select screen with locked/unlocked levels"
```

---

## Task 7: Create 3 new levels (5 total)

**Files:**
- Create: `assets/levels/level_03.ron`
- Create: `assets/levels/level_04.ron`
- Create: `assets/levels/level_05.ron`
- Modify: `src/level.rs` (update LEVELS array)

**Step 1: Update LEVELS array in level.rs**

```rust
pub const LEVELS: &[&str] = &[
    "assets/levels/level_01.ron",
    "assets/levels/level_02.ron",
    "assets/levels/level_03.ron",
    "assets/levels/level_04.ron",
    "assets/levels/level_05.ron",
];
```

**Step 2: Create level_03.ron — "Gaps & Spikes"**

More gaps over void, spikes placed on landing spots. Medium difficulty.

```ron
LevelData(
    name: "Gaps and Spikes",
    spawn: (1.0, 2.0),
    exit: (32.0, 6.0),
    tiles: [
        // Starting ground
        TileEntry(x: 0, y: 0, kind: Platform),
        TileEntry(x: 1, y: 0, kind: Platform),
        TileEntry(x: 2, y: 0, kind: Platform),
        // Jump over gap to spike+platform combo
        TileEntry(x: 5, y: 0, kind: Spikes),
        TileEntry(x: 6, y: 0, kind: Platform),
        TileEntry(x: 7, y: 0, kind: Platform),
        // Rising with spikes
        TileEntry(x: 10, y: 2, kind: Platform),
        TileEntry(x: 11, y: 2, kind: Spikes),
        TileEntry(x: 12, y: 2, kind: Platform),
        // Big gap
        TileEntry(x: 16, y: 3, kind: Platform),
        TileEntry(x: 17, y: 3, kind: Platform),
        // Descend with traps
        TileEntry(x: 20, y: 1, kind: Platform),
        TileEntry(x: 21, y: 1, kind: Spikes),
        TileEntry(x: 22, y: 1, kind: Platform),
        // Final climb
        TileEntry(x: 25, y: 3, kind: Platform),
        TileEntry(x: 26, y: 3, kind: Platform),
        TileEntry(x: 29, y: 5, kind: Platform),
        TileEntry(x: 30, y: 5, kind: Platform),
        TileEntry(x: 31, y: 5, kind: Platform),
        TileEntry(x: 32, y: 5, kind: Platform),
    ],
)
```

**Step 3: Create level_04.ron — "Vertical Climb"**

Narrow platforms stacked vertically. Precision jumping required.

```ron
LevelData(
    name: "Vertical Climb",
    spawn: (1.0, 2.0),
    exit: (18.0, 16.0),
    tiles: [
        // Base
        TileEntry(x: 0, y: 0, kind: Platform),
        TileEntry(x: 1, y: 0, kind: Platform),
        TileEntry(x: 2, y: 0, kind: Platform),
        // Staircase up
        TileEntry(x: 4, y: 2, kind: Platform),
        TileEntry(x: 5, y: 2, kind: Platform),
        TileEntry(x: 7, y: 4, kind: Platform),
        TileEntry(x: 3, y: 6, kind: Platform),
        TileEntry(x: 4, y: 6, kind: Platform),
        // Spike gauntlet
        TileEntry(x: 6, y: 8, kind: Spikes),
        TileEntry(x: 7, y: 8, kind: Platform),
        TileEntry(x: 8, y: 8, kind: Platform),
        // More climbing
        TileEntry(x: 10, y: 10, kind: Platform),
        TileEntry(x: 11, y: 10, kind: Platform),
        TileEntry(x: 13, y: 12, kind: Platform),
        TileEntry(x: 14, y: 12, kind: Platform),
        // Summit with spikes
        TileEntry(x: 15, y: 14, kind: Spikes),
        TileEntry(x: 16, y: 14, kind: Platform),
        TileEntry(x: 17, y: 14, kind: Platform),
        TileEntry(x: 18, y: 14, kind: Platform),
        // Exit platform
        TileEntry(x: 17, y: 15, kind: Platform),
        TileEntry(x: 18, y: 15, kind: Platform),
    ],
)
```

**Step 4: Create level_05.ron — "The Finale"**

Long level combining all mechanics. Gaps, spikes, climbs, descents.

```ron
LevelData(
    name: "The Finale",
    spawn: (1.0, 2.0),
    exit: (45.0, 10.0),
    tiles: [
        // Start
        TileEntry(x: 0, y: 0, kind: Platform),
        TileEntry(x: 1, y: 0, kind: Platform),
        TileEntry(x: 2, y: 0, kind: Platform),
        TileEntry(x: 3, y: 0, kind: Platform),
        // Spike run
        TileEntry(x: 4, y: 0, kind: Spikes),
        TileEntry(x: 5, y: 0, kind: Spikes),
        TileEntry(x: 6, y: 0, kind: Platform),
        TileEntry(x: 7, y: 0, kind: Platform),
        // Gap + climb
        TileEntry(x: 10, y: 2, kind: Platform),
        TileEntry(x: 11, y: 2, kind: Platform),
        TileEntry(x: 13, y: 4, kind: Platform),
        TileEntry(x: 14, y: 4, kind: Platform),
        TileEntry(x: 16, y: 6, kind: Platform),
        TileEntry(x: 17, y: 6, kind: Platform),
        // Descent with spikes
        TileEntry(x: 19, y: 4, kind: Spikes),
        TileEntry(x: 20, y: 4, kind: Platform),
        TileEntry(x: 21, y: 4, kind: Platform),
        TileEntry(x: 22, y: 2, kind: Platform),
        TileEntry(x: 23, y: 2, kind: Platform),
        // Flat spike field
        TileEntry(x: 25, y: 2, kind: Spikes),
        TileEntry(x: 26, y: 2, kind: Platform),
        TileEntry(x: 27, y: 2, kind: Spikes),
        TileEntry(x: 28, y: 2, kind: Platform),
        // Second climb
        TileEntry(x: 30, y: 4, kind: Platform),
        TileEntry(x: 31, y: 4, kind: Platform),
        TileEntry(x: 33, y: 6, kind: Platform),
        TileEntry(x: 34, y: 6, kind: Platform),
        TileEntry(x: 36, y: 8, kind: Platform),
        TileEntry(x: 37, y: 8, kind: Platform),
        // Final gauntlet
        TileEntry(x: 39, y: 8, kind: Spikes),
        TileEntry(x: 40, y: 8, kind: Platform),
        TileEntry(x: 41, y: 8, kind: Spikes),
        TileEntry(x: 42, y: 8, kind: Platform),
        // Exit platform
        TileEntry(x: 43, y: 9, kind: Platform),
        TileEntry(x: 44, y: 9, kind: Platform),
        TileEntry(x: 45, y: 9, kind: Platform),
    ],
)
```

**Step 5: Verify**

Run: `cargo run`
Expected: all 5 levels load. Level select shows 5 levels.

**Step 6: Commit**

```bash
git add src/level.rs assets/levels/
git commit -m "feat: add 3 new levels (5 total)"
```

---

## Task 8: Polish and final verification

**Files:**
- Modify: various

**Step 1: Run clippy**

Run: `cargo clippy -- -W clippy::all`
Fix any warnings.

**Step 2: Full game flow test**

Verify:
1. Menu shows English text: Play / Settings / Quit
2. Settings: English labels, Save button, change value → Save turns green, save it → dim
3. Play → Level Select: level 1 open, others locked
4. Play level 1: smooth movement on platforms (no jerk)
5. Complete level 1 → back to level select, level 2 now unlocked
6. Complete all 5 → back to menu
7. Restart app → progress saved, levels still unlocked
8. Pause works: Resume / Settings / Main Menu in English

**Step 3: Commit**

```bash
git add -A
git commit -m "chore: clippy fixes and final polish"
```

---

## Summary

| # | Task | Key files | Type |
|---|------|-----------|------|
| 1 | Translate UI to English | menu.rs, pause.rs, settings.rs | Bug fix |
| 2 | Fix jerky movement | player.rs | Bug fix |
| 3 | Settings Save button | settings.rs | Bug fix |
| 4 | Add save system deps | Cargo.toml | Setup |
| 5 | Progress save/load | progress.rs, level.rs | Feature |
| 6 | Level select screen | level_select.rs, states.rs, menu.rs | Feature |
| 7 | 3 new levels | assets/levels/, level.rs | Feature |
| 8 | Polish & verify | various | Polish |

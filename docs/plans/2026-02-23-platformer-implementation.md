# SimplePlatformer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a 2D platformer with menu, settings, pause, level loading from RON files, and player movement with avian2d physics.

**Architecture:** Flat module structure in `src/`. Each module is a Bevy plugin registered in `main.rs`. Game flow controlled by `GameState` enum with `DespawnOnExit` for automatic UI cleanup.

**Tech Stack:** Rust, Bevy 0.18 (feature `2d`), avian2d 0.5 (physics), serde + ron (level files)

**Design doc:** `docs/plans/2026-02-23-platformer-design.md`

---

## Task 1: Dependencies and project setup

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add dependencies to Cargo.toml**

```toml
[dependencies]
bevy = { version = "0.18", default-features = false, features = ["2d"] }
avian2d = "0.5"
serde = { version = "1", features = ["derive"] }
ron = "0.11"
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: compiles without errors (downloads new crates)

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add avian2d, serde, ron dependencies"
```

---

## Task 2: Game states

**Files:**
- Create: `src/states.rs`
- Modify: `src/main.rs`

**Step 1: Create `src/states.rs`**

```rust
use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    Settings,
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
```

**Step 2: Wire into `main.rs`**

Replace `main.rs` content — add `mod states;` and register `StatesPlugin`. Keep existing `WindowPlugin` config.

```rust
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
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: compiles without errors

**Step 4: Commit**

```bash
git add src/states.rs src/main.rs
git commit -m "feat: add GameState enum and StatesPlugin"
```

---

## Task 3: Main menu UI

**Files:**
- Create: `src/menu.rs`
- Modify: `src/main.rs`

**Step 1: Create `src/menu.rs`**

Implement main menu with 3 buttons: Начать игру, Настройки, Выйти.

Key details:
- `MenuPlugin` registers `OnEnter(GameState::Menu)` → `spawn_main_menu` system
- Each button entity gets `DespawnOnExit::<GameState>(GameState::Menu)` for automatic cleanup
- `MenuItem` enum component on buttons: `StartGame`, `Settings`, `Exit`
- `SelectedItem` resource (index) for keyboard navigation
- `menu_navigation` system: `↑`/`↓` changes `SelectedItem`, `Enter` activates
- `menu_highlight` system: selected button gets bright color, others dim
- `menu_action` system: on activation — `StartGame` → `NextState(Playing)`, `Settings` → set `SettingsOrigin::Menu` + `NextState(Settings)`, `Exit` → `AppExit` event
- Use Bevy UI `Node` with flexbox for centering: `justify_content: JustifyContent::Center`, `align_items: AlignItems::Center`, `flex_direction: FlexDirection::Column`
- Title text "SIMPLE PLATFORMER" at top, buttons below with gaps

**Step 2: Register in `main.rs`**

Add `mod menu;` and `.add_plugins(menu::MenuPlugin)`.

**Step 3: Run and verify**

Run: `cargo run`
Expected: window opens with main menu. ↑/↓ navigates buttons, Enter on "Выйти" closes app. "Начать игру" transitions to Playing (blank screen — OK for now).

**Step 4: Commit**

```bash
git add src/menu.rs src/main.rs
git commit -m "feat: add main menu with keyboard navigation"
```

---

## Task 4: Settings screen

**Files:**
- Create: `src/settings.rs`
- Modify: `src/main.rs`

**Step 1: Create settings resource**

```rust
#[derive(Resource)]
pub struct GameSettings {
    pub music_volume: f32,     // 0.0 - 1.0
    pub sfx_volume: f32,       // 0.0 - 1.0
    pub resolution: (u32, u32), // (1280, 720) or (1920, 1080)
    pub fullscreen: bool,
}
```

Default: music 0.7, sfx 0.7, 1280x720, windowed.

**Step 2: Create settings UI**

`SettingsPlugin` registers `OnEnter(GameState::Settings)` → `spawn_settings_ui`:
- Title "НАСТРОЙКИ"
- Rows: Музыка [===----] 70%, Звуки [===----] 70%, Разрешение: 1280x720 / 1920x1080 toggle, Окно: Оконный / Полный экран toggle, Назад
- `←`/`→` adjusts values, `↑`/`↓` navigates rows, `Enter` on "Назад" or `Esc` returns
- On return: check `SettingsOrigin` → go to `Menu` or `Paused`
- Apply resolution/fullscreen changes immediately to `Window` entity
- All UI entities get `DespawnOnExit::<GameState>(GameState::Settings)`

**Step 3: Register in `main.rs`**

Add `mod settings;` and `.add_plugins(settings::SettingsPlugin)`.

**Step 4: Run and verify**

Run: `cargo run`
Expected: Menu → Настройки shows settings screen. Can change values. "Назад" returns to menu.

**Step 5: Commit**

```bash
git add src/settings.rs src/main.rs
git commit -m "feat: add settings screen with volume, resolution, fullscreen"
```

---

## Task 5: Physics setup

**Files:**
- Create: `src/physics.rs`
- Modify: `src/main.rs`

**Step 1: Create `src/physics.rs`**

```rust
use avian2d::prelude::*;
use bevy::prelude::*;

use crate::states::GameState;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(avian2d::PhysicsPlugins::default())
            .insert_resource(Gravity(Vec2::new(0.0, -980.0)))
            .add_systems(Update, pause_physics.run_if(not(in_state(GameState::Playing))));
    }
}

fn pause_physics(mut physics_time: ResMut<Time<Physics>>) {
    physics_time.pause();
}
```

Note: physics should only run during `Playing`. Use `Time<Physics>` to pause/unpause. Add a corresponding `unpause_physics` system that runs `OnEnter(GameState::Playing)`.

**Step 2: Register in `main.rs`**

Add `mod physics;` and `.add_plugins(physics::PhysicsPlugin)`.

**Step 3: Verify compilation**

Run: `cargo check`
Expected: compiles (avian2d plugin integrates with Bevy)

**Step 4: Commit**

```bash
git add src/physics.rs src/main.rs
git commit -m "feat: add avian2d physics with gravity, pause when not playing"
```

---

## Task 6: Player module

**Files:**
- Create: `src/player.rs`
- Modify: `src/main.rs`

**Step 1: Define player components**

```rust
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Grounded(pub bool);

#[derive(Resource)]
pub struct SpawnPoint(pub Vec2);
```

**Step 2: Create `spawn_player` system**

Runs on `OnEnter(GameState::Playing)`:
- Spawn entity with: `Player`, `Grounded(false)`, `Sprite` (blue rectangle 24x32), `RigidBody::Dynamic`, `Collider::rectangle(24.0, 32.0)`, `LockedAxes::ROTATION_LOCKED`, `LinearVelocity::ZERO`, `Transform` at `SpawnPoint` position
- The `SpawnPoint` resource is set by level loading (Task 8)

**Step 3: Create `player_movement` system**

Runs in `Update` when `in_state(GameState::Playing)`:
- Read `KeyCode` input (A/Left, D/Right)
- Set horizontal `LinearVelocity.x`: ±300.0 when pressed, 0.0 when released
- Space pressed + `Grounded(true)` → set `LinearVelocity.y = 500.0` (jump impulse)

**Step 4: Create `ground_detection` system**

Use avian2d `ShapeHits` or raycasting downward from player to detect ground:
- Cast a short ray (or shape) from player's feet
- Update `Grounded` component

**Step 5: Create `player_death` system**

Runs in `Update` when `in_state(GameState::Playing)`:
- If player `Transform.translation.y < -500.0` (fell off map) → teleport to `SpawnPoint`
- Collision with `Spikes` entity (detected via avian2d collision events) → teleport to `SpawnPoint`

**Step 6: Register in `main.rs`**

Add `mod player;` and `.add_plugins(player::PlayerPlugin)`.

**Step 7: Verify compilation**

Run: `cargo check`
Expected: compiles (player won't appear yet — needs level to set SpawnPoint)

**Step 8: Commit**

```bash
git add src/player.rs src/main.rs
git commit -m "feat: add player with movement, jump, ground detection, death"
```

---

## Task 7: Camera module

**Files:**
- Create: `src/camera.rs`
- Modify: `src/main.rs`

**Step 1: Create `src/camera.rs`**

```rust
pub struct CameraPlugin;
```

Systems:
- `spawn_camera`: on `OnEnter(GameState::Playing)` → spawn `Camera2d` entity
- `camera_follow`: in `Update` when `Playing` → lerp camera `Transform` toward player position. `let speed = 5.0; camera.translation = camera.translation.lerp(player.translation.extend(0.0), speed * time.delta_secs());` Keep camera z at a fixed value (e.g., 0.0 for 2D).

**Step 2: Register in `main.rs`**

Add `mod camera;` and `.add_plugins(camera::CameraPlugin)`.

**Step 3: Verify compilation**

Run: `cargo check`

**Step 4: Commit**

```bash
git add src/camera.rs src/main.rs
git commit -m "feat: add camera with smooth player follow"
```

---

## Task 8: Level loading system

**Files:**
- Create: `src/level.rs`
- Create: `assets/levels/level_01.ron`
- Create: `assets/levels/level_02.ron`
- Modify: `src/main.rs`

**Step 1: Define level data structures**

```rust
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LevelData {
    pub name: String,
    pub spawn: (f32, f32),
    pub exit: (f32, f32),
    pub tiles: Vec<TileEntry>,
}

#[derive(Deserialize)]
pub struct TileEntry {
    pub x: i32,
    pub y: i32,
    pub kind: TileKind,
}

#[derive(Deserialize, Clone, Copy)]
pub enum TileKind {
    Platform,
    Spikes,
}
```

**Step 2: Define ECS components for tiles**

```rust
#[derive(Component)]
pub struct Platform;

#[derive(Component)]
pub struct Spikes;

#[derive(Component)]
pub struct Exit;

#[derive(Resource)]
pub struct CurrentLevel(pub usize);

const TILE_SIZE: f32 = 32.0;
const LEVELS: &[&str] = &["levels/level_01.ron", "levels/level_02.ron"];
```

**Step 3: Create `load_level` system**

Runs on `OnEnter(GameState::Playing)`:
- Read RON file from `assets/levels/` using `std::fs::read_to_string` (simple approach for learning project, no async asset loading needed)
- Parse with `ron::from_str::<LevelData>`
- Set `SpawnPoint` resource from level data
- For each tile: spawn `Sprite` (colored rectangle) + `Collider` + marker component:
  - `Platform` → gray 32x32, `RigidBody::Static`, `Collider::rectangle(TILE_SIZE, TILE_SIZE)`
  - `Spikes` → red 32x32, `RigidBody::Static`, `Collider::rectangle(TILE_SIZE, TILE_SIZE)`, sensor (no physical push)
- Spawn exit entity: green 32x32, `RigidBody::Static`, `Collider::rectangle(TILE_SIZE, TILE_SIZE)`, sensor
- All level entities get `DespawnOnExit::<GameState>(GameState::Playing)` for cleanup

**Step 4: Create `check_exit` system**

In `Update` when `Playing`:
- Detect collision between `Player` and `Exit` entity (avian2d collision events)
- Increment `CurrentLevel`
- If more levels → `NextState(Playing)` (re-enters, triggers cleanup + load)
- If no more levels → `NextState(Menu)`

**Step 5: Create level files**

`assets/levels/level_01.ron`:
```ron
LevelData(
    name: "Начало",
    spawn: (2.0, 5.0),
    exit: (25.0, 5.0),
    tiles: [
        // Ground floor
        TileEntry(x: 0, y: 0, kind: Platform),
        TileEntry(x: 1, y: 0, kind: Platform),
        TileEntry(x: 2, y: 0, kind: Platform),
        TileEntry(x: 3, y: 0, kind: Platform),
        TileEntry(x: 4, y: 0, kind: Platform),
        TileEntry(x: 5, y: 0, kind: Platform),
        // Gap (player must jump)
        TileEntry(x: 8, y: 0, kind: Platform),
        TileEntry(x: 9, y: 0, kind: Platform),
        TileEntry(x: 10, y: 0, kind: Platform),
        // Spikes
        TileEntry(x: 11, y: 0, kind: Spikes),
        // More platforms
        TileEntry(x: 14, y: 2, kind: Platform),
        TileEntry(x: 15, y: 2, kind: Platform),
        TileEntry(x: 18, y: 4, kind: Platform),
        TileEntry(x: 19, y: 4, kind: Platform),
        TileEntry(x: 22, y: 4, kind: Platform),
        TileEntry(x: 23, y: 4, kind: Platform),
        TileEntry(x: 24, y: 4, kind: Platform),
        TileEntry(x: 25, y: 4, kind: Platform),
    ],
)
```

`assets/levels/level_02.ron` — similar but slightly harder layout.

**Step 6: Register in `main.rs`**

Add `mod level;` and `.add_plugins(level::LevelPlugin)`. Insert `CurrentLevel(0)` as initial resource.

**Step 7: Run and verify full gameplay loop**

Run: `cargo run`
Expected: Menu → Start → level loads, player spawns, can move and jump on platforms. Reaching exit loads next level. After last level → back to menu.

**Step 8: Commit**

```bash
git add src/level.rs assets/levels/ src/main.rs
git commit -m "feat: add level loading from RON files with 2 levels"
```

---

## Task 9: Pause screen

**Files:**
- Modify: `src/menu.rs` (or create `src/pause.rs` — keep in menu.rs for simplicity)

**Step 1: Add pause toggle system**

In `Update` when `Playing`: if `Esc` just pressed → `NextState(Paused)`.
In `Update` when `Paused`: if `Esc` just pressed → `NextState(Playing)`.

**Step 2: Create pause overlay UI**

On `OnEnter(GameState::Paused)`:
- Spawn fullscreen semi-transparent black overlay (`BackgroundColor` with alpha 0.7)
- Center panel with title "ПАУЗА"
- Buttons: Продолжить / Настройки / В меню
- Same keyboard navigation as main menu
- Actions: Продолжить → `NextState(Playing)`, Настройки → `SettingsOrigin::Paused` + `NextState(Settings)`, В меню → `NextState(Menu)`
- All entities get `DespawnOnExit::<GameState>(GameState::Paused)`

**Step 3: Ensure physics pauses**

Already handled in Task 5 — physics only runs in `Playing` state.

**Step 4: Run and verify**

Run: `cargo run`
Expected: During gameplay, Esc shows pause overlay. Can resume, go to settings, or return to menu. Game world visible behind dark overlay.

**Step 5: Commit**

```bash
git add src/menu.rs
git commit -m "feat: add pause screen with overlay"
```

---

## Task 10: Camera for menu (fix)

**Files:**
- Modify: `src/camera.rs`

**Step 1: Ensure camera exists for UI**

The menu needs a camera to render. Move camera spawn to app startup (not `OnEnter(Playing)`). The camera should always exist. The `camera_follow` system should only run during `Playing`.

**Step 2: Run full game loop**

Run: `cargo run`
Expected: Menu renders correctly → Play → camera follows player → Pause → Resume → Exit level → back to menu. Full loop works.

**Step 3: Commit**

```bash
git add src/camera.rs
git commit -m "fix: spawn camera at startup so menu renders correctly"
```

---

## Task 11: Polish and final verification

**Files:**
- Modify: various (minor tweaks)

**Step 1: Run clippy**

Run: `cargo clippy -- -W clippy::all`
Expected: no warnings. Fix any that appear.

**Step 2: Test full game flow**

Verify manually:
1. App opens → main menu visible
2. ↑/↓ navigates menu, Enter selects
3. "Настройки" → settings screen, values adjustable, "Назад" returns
4. "Начать игру" → level 1 loads, player visible
5. A/D moves, Space jumps, physics works
6. Falling off → respawn
7. Touching spikes → respawn
8. Reaching exit → level 2 loads
9. Completing level 2 → back to menu
10. During gameplay, Esc → pause overlay
11. Pause: Продолжить resumes, Настройки opens settings (returns to pause), В меню goes to menu
12. "Выйти" from menu → app closes

**Step 3: Final commit**

```bash
git add -A
git commit -m "chore: clippy fixes and polish"
```

---

## Summary of tasks

| # | Task | Key files | Depends on |
|---|------|-----------|------------|
| 1 | Dependencies | Cargo.toml | — |
| 2 | Game states | states.rs, main.rs | 1 |
| 3 | Main menu | menu.rs | 2 |
| 4 | Settings screen | settings.rs | 2, 3 |
| 5 | Physics setup | physics.rs | 1 |
| 6 | Player | player.rs | 2, 5 |
| 7 | Camera | camera.rs | 2, 6 |
| 8 | Level loading | level.rs, assets/ | 5, 6 |
| 9 | Pause screen | menu.rs | 3, 5 |
| 10 | Camera fix | camera.rs | 3, 7 |
| 11 | Polish | various | all |

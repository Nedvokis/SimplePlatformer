# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SimplePlatformer — 2D side-scrolling platformer built with Rust and Bevy 0.18 + avian2d 0.5 for physics. Player jumps across platforms to reach the level exit. Uses Bevy's `2d` feature set (default features disabled). 5 levels with progress saving.

## Commands

```bash
cargo run              # Run the game
cargo build --release  # Release build
cargo check            # Fast compilation check
cargo clippy           # Linting
cargo test             # Run tests
```

## Architecture

Bevy ECS architecture: systems operate on components attached to entities. The app is configured in `src/main.rs` as an `App` with plugins and systems.

### Plugins (src/)

| File | Plugin | Purpose |
|------|--------|---------|
| `states.rs` | — | Game states enum + SettingsOrigin resource |
| `menu.rs` | MenuPlugin | Main menu (Play/Settings/Quit) |
| `level_select.rs` | LevelSelectPlugin | Level selection with lock/unlock |
| `level.rs` | LevelPlugin | Level loading from RON, merged colliders, exit/spikes |
| `player.rs` | PlayerPlugin | Player spawn, movement (FixedUpdate), ground detection |
| `camera.rs` | CameraPlugin | Smooth camera follow |
| `physics.rs` | PhysicsPlugin | avian2d setup, gravity, pause/unpause |
| `pause.rs` | PausePlugin | Pause overlay (Esc toggle) |
| `settings.rs` | SettingsPlugin | Settings screen, reset progress popup |
| `progress.rs` | — | PlayerProgress save/load (JSON to ~/.local/share/) |

### Game States

```
Menu → LevelSelect → LevelTransition → Playing ⇄ Paused
                                          ↓         ↓
                                       Settings ← Settings
```

### Key Patterns

- Use `States` for game states, `DespawnOnExit` for cleanup
- Use `Component` derive for entity data, `Resource` for global data
- Player movement runs in `FixedUpdate` (synchronized with physics solver)
- Platform colliders are merged horizontally to prevent ghost collisions at tile seams
- UI buttons use Bevy's `Button` + `Interaction` for mouse support
- `SettingsRow(index)` identifies UI rows; popup buttons use index >= 100
- `GlobalZIndex` for overlay layering (100 = pause, 200 = reset popup)
- Level files in RON format: `assets/levels/level_NN.ron`

## Game Design

- **Main menu** at launch: Play / Settings / Quit
- **Level select**: 5 levels, locked until previous completed
- **Controls**: A/← left, D/→ right, Space jump, Esc pause
- **Mouse**: hover highlights, click activates (all menus)
- **Window**: 1280×720, title "SimplePlatformer"
- **Progress**: auto-saved to JSON on level completion

## Conventions

- Language in code: English
- Language in UI: English
- Language in docs: Russian
- Bevy 0.18 API — `WindowResolution` takes `u32` tuples, not floats
- Bevy 0.18: `BorderRadius` is a field on `Node`, not a separate component
- Bevy 0.18: `despawn()` is already recursive, no `despawn_recursive()`
- Dev profile: `opt-level = 1` for game code, `opt-level = 3` for dependencies (faster iteration)

## Known Bevy 0.18 Gotchas

- `BorderRadius` must be set via `Node { border_radius: BorderRadius::all(...), .. }`, not as a tuple component
- `despawn_recursive()` doesn't exist — `despawn()` already removes children
- Player systems must run in `FixedUpdate` to stay synchronized with avian2d physics solver
- Each tile having its own collider causes ghost collisions — merge adjacent tiles into single colliders

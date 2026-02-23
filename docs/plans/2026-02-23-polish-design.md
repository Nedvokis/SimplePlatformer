# SimplePlatformer Polish & Features — Design

**Date:** 2026-02-23
**Source:** docs/polish_work/PROBLEMS.md

---

## Bug Fixes

### 1. Tofu text (squares instead of characters)
All Russian UI text displays as squares because the default Bevy font lacks Cyrillic glyphs.
**Fix:** Translate all UI text to English. No custom font needed.

### 2. Jerky movement on platforms
Player movement stutters when walking on platforms due to physics friction between player and static colliders.
**Fix:** Set `Friction { dynamic_coefficient: 0.0, static_coefficient: 0.0 }` on the player entity. Horizontal movement is already controlled manually via LinearVelocity.

### 3. Save button in settings
Currently settings apply instantly with no explicit save action.
**Fix:** Add a "Save" button (before "Back") that:
- Is dim/inactive by default
- Highlights green when any setting has been modified (`SettingsChanged(bool)` resource)
- On activation: confirms save, resets the changed flag
- Settings still apply immediately (save is for user feedback)

## New Features

### 4. Level select screen
New `GameState::LevelSelect` state. Flow: Menu → "Play" → LevelSelect → choose level → Playing.

UI: Grid of numbered level buttons. Level 1 always unlocked. Locked levels shown dimmed with "Locked" text. Arrow navigation + Enter. "Back" returns to Menu.

### 5. Save system (player progress)
Save `max_unlocked_level` to `~/.local/share/simple_platformer/save.json`.
- Load on app startup into `PlayerProgress` resource
- Update and write to disk when a level is completed
- Level select screen reads `PlayerProgress` for lock state
- New dependency: `serde_json` for JSON serialization, `dirs` for platform paths

### 6. Three new levels (5 total)
- Level 3: More gaps, spikes on platforms
- Level 4: Narrow platforms, vertical climbing
- Level 5: Long finale combining all mechanics

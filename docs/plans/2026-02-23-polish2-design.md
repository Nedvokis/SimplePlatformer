# SimplePlatformer Polish Round 2 — Design

**Date:** 2026-02-23
**Source:** docs/polish_work/PROBLEMS_2.md

---

## Bug Fixes

### 1. Level completion sends player to menu instead of next level
`check_exit` transitions from `Playing` to `Playing`, which doesn't trigger `OnEnter`/`OnExit` so the next level never loads.
**Fix:** Add transient `GameState::LevelTransition` state. `check_exit` sets `LevelTransition`, which immediately transitions back to `Playing`, forcing full entity despawn and reload. After the last level, go to `LevelSelect`.

### 2. Player still moves jerkily on platforms
`Friction::ZERO` was added to the player but not to platform colliders. avian2d computes contact friction from both bodies.
**Fix:** Add `Friction::ZERO` to platform tile spawns in `load_level`.

### 3. All levels unlock after completing level 1
Root cause is the same as bug 1. The `Playing` -> `Playing` non-transition leaves old exit entities alive, so `check_exit` fires every frame, incrementing `max_unlocked_level` repeatedly.
**Fix:** Automatically resolved by fix 1.

## Improvements

### 4 & 5. Reset Progress confirmation should be an overlay popup with real buttons
Current implementation replaces text in the settings row. User wants a proper overlay popup (like the pause menu) with separate Yes/No buttons to prevent accidental resets.

**Design:**
- Press Enter on "Reset Progress" -> spawn dark semi-transparent overlay
- Overlay contains: "Reset Progress?" title, "All level progress will be lost." warning, and two buttons: "Yes" / "No"
- Arrow key navigation between Yes/No, Enter to confirm
- Esc or "No" closes popup without resetting
- "Yes" resets progress, saves, closes popup
- Settings navigation is disabled while popup is open
- Remove the old inline text-replacement approach

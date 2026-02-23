# SimplePlatformer Polish Round 3 — Design

**Date:** 2026-02-23
**Source:** docs/polish_work/PROBLEMS_3.md

---

## Bug Fixes

### 1. Jerky movement on platforms (still happening)
Player movement stutters when walking on platforms despite Friction::ZERO on both player and platforms.
**Root cause hypothesis:** `player_movement` runs in `Update` but physics solver runs in `FixedUpdate`. The velocity we set gets overwritten by the physics solver between frames.
**Approach:** Debug first — add temporary logging to `player_movement` (Update) and a FixedUpdate system to see what physics does to velocity. Analyze output. Most likely fix: move `player_movement` and `ground_detection` to `FixedUpdate` to sync with the physics solver.

### 2. Reset Progress popup looks broken
The overlay panel has no background — text floats over the settings screen. Yes/No buttons show as colored rectangles with no visible text.
**Fix:** Add `BackgroundColor` and `padding` to the panel Node so it's opaque and text is readable.

## Improvements

### 3. Mouse support for menus
All menus currently use keyboard-only navigation. Add mouse hover (highlight) and click (activate) support.
**Approach:** Bevy's `Button` component already tracks `Interaction` (Hovered/Pressed). Add a system per screen that reads `Interaction`, syncs with the `SelectedItem` resource on hover, and triggers the action on press.
**Screens:** Menu, Settings, Pause, Level Select, Reset Popup (Yes/No buttons).

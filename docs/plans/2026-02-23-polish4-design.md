# SimplePlatformer Polish Round 4 — Design

## Problem 1: Jerky movement on platforms (ghost collisions)

### Root cause

Each platform tile has its own 32x32 collider. When the player moves horizontally across adjacent tiles, the physics solver catches on the seams between neighboring colliders — a classic "ghost collision" / "tile seam" problem. This causes the player to stutter on platforms while movement in the air is smooth.

### Solution: Merge adjacent platform colliders

During level loading, group horizontally adjacent Platform tiles into strips. Each strip gets a single wide collider instead of individual per-tile colliders.

**Algorithm:**
1. Collect all Platform tile positions into a `HashSet<(i32, i32)>`
2. For each unique y-coordinate, find contiguous horizontal runs of tiles
3. Each run of N tiles → one `RigidBody::Static` + `Collider::rectangle(N * TILE_SIZE, TILE_SIZE)` centered on the run
4. Individual tile sprites remain unchanged (one per tile) for visual rendering
5. Collider entities have no Sprite — they are invisible physics-only entities

**Files:** `src/level.rs`

## Problem 2: Popup buttons show no text

### Root cause

`settings_update_text` iterates all `SettingsRow` entities and calls `row_text(row.0, ...)`. For popup buttons (row.0 = 100, 101), `row_text` returns `""` from the `_ => String::new()` branch. This overwrites the "Yes"/"No" text every frame.

### Solution

Skip popup rows in `settings_update_text`: add `if row.0 >= 100 { continue; }` at the top of the loop.

**Files:** `src/settings.rs`

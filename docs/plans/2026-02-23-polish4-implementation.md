# SimplePlatformer Polish Round 4 — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix ghost collisions on tile seams by merging adjacent platform colliders, and fix invisible popup button text.

**Architecture:** Refactor level loading to separate visual sprites from physics colliders. Adjacent Platform tiles on the same row get one merged collider. Fix `settings_update_text` to skip popup button rows.

**Tech Stack:** Rust, Bevy 0.18, avian2d 0.5

**Design doc:** `docs/plans/2026-02-23-polish4-design.md`

---

## Task 1: Merge adjacent platform colliders

**Files:**
- Modify: `src/level.rs`

**Step 1: Add helper function to find horizontal runs**

Add this function before `load_level`:

```rust
use std::collections::HashSet;

/// Groups platform tiles into horizontal runs for merged colliders.
/// Returns a list of (start_x, y, count) tuples.
fn merge_platform_runs(tiles: &[TileEntry]) -> Vec<(i32, i32, usize)> {
    let platforms: HashSet<(i32, i32)> = tiles
        .iter()
        .filter(|t| matches!(t.kind, TileKind::Platform))
        .map(|t| (t.x, t.y))
        .collect();

    let mut runs = Vec::new();
    let mut visited: HashSet<(i32, i32)> = HashSet::new();

    // Get sorted unique y values
    let mut ys: Vec<i32> = platforms.iter().map(|(_, y)| *y).collect();
    ys.sort();
    ys.dedup();

    for y in ys {
        // Get sorted x values for this row
        let mut xs: Vec<i32> = platforms
            .iter()
            .filter(|(_, py)| *py == y)
            .map(|(x, _)| *x)
            .collect();
        xs.sort();

        let mut i = 0;
        while i < xs.len() {
            let start_x = xs[i];
            if visited.contains(&(start_x, y)) {
                i += 1;
                continue;
            }
            let mut count = 1;
            while i + count < xs.len() && xs[i + count] == start_x + count as i32 {
                count += 1;
            }
            for j in 0..count {
                visited.insert((start_x + j as i32, y));
            }
            runs.push((start_x, y, count));
            i += count;
        }
    }

    runs
}
```

**Step 2: Refactor `load_level` to use merged colliders**

In `load_level`, replace the `TileKind::Platform` arm of the match. Currently (lines 81–94) each platform tile spawns with both a Sprite AND a Collider. Change it so:

1. Platform tiles spawn with Sprite only (no RigidBody, Collider, or Friction)
2. After the tile loop, spawn merged collider entities

Replace the entire tile spawning section. The new `load_level` body after the spawn point setup should be:

```rust
    // Spawn tile sprites (visual only)
    for tile in &level.tiles {
        let pos = Vec3::new(tile.x as f32 * TILE_SIZE, tile.y as f32 * TILE_SIZE, 0.0);
        match tile.kind {
            TileKind::Platform => {
                commands.spawn((
                    Platform,
                    Sprite {
                        color: Color::srgb(0.4, 0.4, 0.4),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_translation(pos),
                    DespawnOnExit::<GameState>(GameState::Playing),
                ));
            }
            TileKind::Spikes => {
                commands.spawn((
                    Spikes,
                    Sprite {
                        color: Color::srgb(0.9, 0.2, 0.2),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_translation(pos),
                    RigidBody::Static,
                    Collider::rectangle(TILE_SIZE, TILE_SIZE),
                    Sensor,
                    CollidingEntities::default(),
                    DespawnOnExit::<GameState>(GameState::Playing),
                ));
            }
        }
    }

    // Spawn merged platform colliders (physics only, no sprite)
    for (start_x, y, count) in merge_platform_runs(&level.tiles) {
        let width = count as f32 * TILE_SIZE;
        let center_x = start_x as f32 * TILE_SIZE + (width - TILE_SIZE) / 2.0;
        let center_y = y as f32 * TILE_SIZE;
        commands.spawn((
            Platform,
            RigidBody::Static,
            Collider::rectangle(width, TILE_SIZE),
            Friction::ZERO,
            Transform::from_xyz(center_x, center_y, 0.0),
            DespawnOnExit::<GameState>(GameState::Playing),
        ));
    }
```

Note: The `use std::collections::HashSet;` import needs to be added at the top of the file.

**Step 3: Verify**

Run: `cargo check`

**Step 4: Commit**

```bash
git add src/level.rs
git commit -m "fix: merge adjacent platform colliders to eliminate ghost collisions

Platform tiles now spawn sprites separately from physics colliders.
Adjacent tiles on the same row share one wide collider, eliminating
seam collisions that caused jerky movement.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 2: Fix popup button text being overwritten

**Files:**
- Modify: `src/settings.rs`

**Step 1: Skip popup rows in `settings_update_text`**

In `settings_update_text` (line 463), add an early continue for popup button rows. Change:

```rust
fn settings_update_text(
    settings: Res<GameSettings>,
    rows: Query<(&SettingsRow, &Children)>,
    mut texts: Query<&mut Text>,
) {
    for (row, children) in &rows {
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                **text = row_text(row.0, &settings);
            }
        }
    }
}
```

to:

```rust
fn settings_update_text(
    settings: Res<GameSettings>,
    rows: Query<(&SettingsRow, &Children)>,
    mut texts: Query<&mut Text>,
) {
    for (row, children) in &rows {
        if row.0 >= 100 {
            continue;
        }
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                **text = row_text(row.0, &settings);
            }
        }
    }
}
```

**Step 2: Verify**

Run: `cargo check`

**Step 3: Commit**

```bash
git add src/settings.rs
git commit -m "fix: prevent popup button text from being overwritten

settings_update_text was calling row_text() on popup buttons (row >= 100),
which returned empty string and erased the Yes/No labels every frame.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 3: Clippy and final verification

**Files:**
- Modify: various (as needed)

**Step 1: Run clippy**

Run: `cargo clippy -- -W clippy::all`
Fix any warnings.

**Step 2: Commit (if changes)**

```bash
git add -A
git commit -m "chore: clippy fixes for polish round 4

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Summary

| # | Task | Key files | Type |
|---|------|-----------|------|
| 1 | Merge adjacent platform colliders | level.rs | Bug fix |
| 2 | Fix popup button text | settings.rs | Bug fix |
| 3 | Clippy & verify | various | Polish |

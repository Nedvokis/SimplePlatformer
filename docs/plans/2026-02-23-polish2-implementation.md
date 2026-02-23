# SimplePlatformer Polish Round 2 — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix level transitions (auto-advance), platform friction, all-levels-unlock bug, and replace inline reset confirmation with overlay popup.

**Architecture:** Add transient `LevelTransition` state to force proper `OnExit`/`OnEnter` cycle between levels. Add `Friction::ZERO` to platform colliders. Replace inline text-based reset confirmation with a spawned overlay popup containing separate Yes/No buttons.

**Tech Stack:** Rust, Bevy 0.18, avian2d 0.5

**Design doc:** `docs/plans/2026-02-23-polish2-design.md`

---

## Task 1: Fix level transitions (auto-advance) and all-levels-unlock bug

**Files:**
- Modify: `src/states.rs`
- Modify: `src/level.rs`

These two bugs share the same root cause: `check_exit` transitions `Playing` → `Playing` which doesn't trigger `OnExit`/`OnEnter`. Old entities persist, `check_exit` fires every frame, incrementing `max_unlocked_level` repeatedly.

**Step 1: Add `LevelTransition` variant to `GameState`**

In `src/states.rs`, add `LevelTransition` between `LevelSelect` and `Playing`:

```rust
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    LevelSelect,
    LevelTransition,
    Playing,
    Paused,
    Settings,
}
```

**Step 2: Add transition system in `src/level.rs`**

In `LevelPlugin::build`, add a system that auto-advances from `LevelTransition` → `Playing`:

```rust
.add_systems(OnEnter(GameState::LevelTransition), transition_to_playing)
```

Implement:
```rust
fn transition_to_playing(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}
```

**Step 3: Update `check_exit` to use `LevelTransition`**

In `check_exit` (currently at line 131–156), change the two state transitions:

- When there are more levels: change `next_state.set(GameState::Playing)` to `next_state.set(GameState::LevelTransition)`
- When all levels are complete: change `next_state.set(GameState::Menu)` to `next_state.set(GameState::LevelSelect)` (and remove `current_level.0 = 0` — let level select handle that)

The updated `check_exit` body should be:
```rust
fn check_exit(
    exit_query: Query<&CollidingEntities, With<Exit>>,
    player_query: Query<(), With<Player>>,
    mut current_level: ResMut<CurrentLevel>,
    mut progress: ResMut<PlayerProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for colliding in &exit_query {
        for &entity in colliding.iter() {
            if player_query.get(entity).is_ok() {
                current_level.0 += 1;
                if current_level.0 > progress.max_unlocked_level {
                    progress.max_unlocked_level = current_level.0;
                    crate::progress::save_progress(&progress);
                }
                if current_level.0 < LEVELS.len() {
                    next_state.set(GameState::LevelTransition);
                } else {
                    next_state.set(GameState::LevelSelect);
                }
                return;
            }
        }
    }
}
```

**Step 4: Verify**

Run: `cargo check`

**Step 5: Commit**

```bash
git add src/states.rs src/level.rs
git commit -m "fix: add LevelTransition state to fix level auto-advance and unlock bug"
```

---

## Task 2: Fix platform friction

**Files:**
- Modify: `src/level.rs`

**Step 1: Add `Friction::ZERO` to platform spawns**

In `load_level`, in the `TileKind::Platform` match arm (around line 80–92), add `Friction::ZERO` to the platform entity tuple. The `Friction` type is from `avian2d::prelude::*` which is already imported.

Add it after the `Collider::rectangle(TILE_SIZE, TILE_SIZE)` line:

```rust
TileKind::Platform => {
    commands.spawn((
        Platform,
        Sprite {
            color: Color::srgb(0.4, 0.4, 0.4),
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(pos),
        RigidBody::Static,
        Collider::rectangle(TILE_SIZE, TILE_SIZE),
        Friction::ZERO,
        DespawnOnExit::<GameState>(GameState::Playing),
    ));
}
```

**Step 2: Verify**

Run: `cargo check`

**Step 3: Commit**

```bash
git add src/level.rs
git commit -m "fix: add zero friction to platform colliders"
```

---

## Task 3: Replace inline reset confirmation with overlay popup

**Files:**
- Modify: `src/settings.rs`

This is the largest task. Replace the inline text replacement ("Are you sure? Yes / No" in the same row) with a proper overlay popup containing separate Yes/No buttons.

**Step 1: Add popup marker component and selected-confirm resource**

```rust
/// Marker for the confirmation overlay root entity.
#[derive(Component)]
struct ResetOverlay;

/// Which confirmation button is selected: 0 = Yes, 1 = No.
#[derive(Resource, Default)]
pub struct SelectedConfirmItem(pub usize);
```

Register `SelectedConfirmItem` in plugin: `.init_resource::<SelectedConfirmItem>()`

**Step 2: Remove `confirming_reset` parameter from `row_text`**

Row 5 should always show "Reset Progress" — the confirmation is now handled by the popup overlay.

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
        4 => "Save".to_string(),
        5 => "Reset Progress".to_string(),
        6 => "Back".to_string(),
        _ => String::new(),
    }
}
```

Update all call sites to remove the `confirming_reset` argument:
- `setup_settings`: `row_text(i, &settings)` (remove `, false`)
- `settings_update_text`: `row_text(row.0, &settings)` (remove `, confirming.0`)

Remove `confirming: Res<ConfirmingReset>` from `settings_update_text` parameters since it's no longer needed there.

**Step 3: Add popup spawn function**

```rust
fn spawn_reset_overlay(commands: &mut Commands, selected_confirm: &mut ResMut<SelectedConfirmItem>) {
    selected_confirm.0 = 1; // Default to "No" for safety

    commands
        .spawn((
            ResetOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(200),
        ))
        .with_children(|overlay| {
            overlay
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("Reset Progress?"),
                        TextFont { font_size: 32.0, ..default() },
                        TextColor(Color::WHITE),
                        Node { margin: UiRect::bottom(Val::Px(10.0)), ..default() },
                    ));

                    // Warning
                    panel.spawn((
                        Text::new("All level progress will be lost."),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.4, 0.4)),
                        Node { margin: UiRect::bottom(Val::Px(30.0)), ..default() },
                    ));

                    // Buttons row
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(20.0),
                            ..default()
                        })
                        .with_children(|row| {
                            for (i, label) in ["Yes", "No"].iter().enumerate() {
                                row.spawn((
                                    Button,
                                    SettingsRow(100 + i), // Use 100+ offset to distinguish from main settings rows
                                    Node {
                                        width: Val::Px(120.0),
                                        height: Val::Px(45.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(COLOR_NORMAL),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new(*label),
                                        TextFont { font_size: 22.0, ..default() },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            }
                        });
                });
        });
}
```

**Step 4: Update `settings_adjust` to spawn/handle popup**

Replace the old inline confirmation logic. The key changes:

When `confirming.0 == true`:
- All input is routed to the popup (navigate Yes/No with Left/Right arrows, confirm with Enter, cancel with Esc)
- Regular settings navigation is blocked

When Enter is pressed on index 5 (Reset Progress) and `confirming.0 == false`:
- Set `confirming.0 = true`
- Call `spawn_reset_overlay(&mut commands, &mut selected_confirm)`

When confirming:
- Left/Right arrows: toggle `selected_confirm.0` between 0 (Yes) and 1 (No)
- Enter: if `selected_confirm.0 == 0` (Yes), reset progress and close overlay; if 1 (No), close overlay
- Esc: close overlay

To close the overlay, despawn all entities with `ResetOverlay` component and set `confirming.0 = false`.

Add `mut commands: Commands`, `mut selected_confirm: ResMut<SelectedConfirmItem>`, and `overlay_query: Query<Entity, With<ResetOverlay>>` to `settings_adjust` parameters.

Here's the updated `settings_adjust` (full replacement):

```rust
#[allow(clippy::too_many_arguments)]
fn settings_adjust(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedSettingsItem>,
    mut settings: ResMut<GameSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    origin: Res<SettingsOrigin>,
    mut windows: Query<&mut Window>,
    mut changed: ResMut<SettingsChanged>,
    mut confirming: ResMut<ConfirmingReset>,
    mut progress: ResMut<PlayerProgress>,
    mut selected_confirm: ResMut<SelectedConfirmItem>,
    overlay_query: Query<Entity, With<ResetOverlay>>,
) {
    // When popup is open, handle popup input only
    if confirming.0 {
        if keyboard.just_pressed(KeyCode::Escape) {
            // Cancel
            for entity in &overlay_query {
                commands.entity(entity).despawn();
            }
            confirming.0 = false;
            return;
        }
        if keyboard.just_pressed(KeyCode::ArrowLeft) {
            selected_confirm.0 = 0; // Yes
        }
        if keyboard.just_pressed(KeyCode::ArrowRight) {
            selected_confirm.0 = 1; // No
        }
        if keyboard.just_pressed(KeyCode::Enter) {
            if selected_confirm.0 == 0 {
                // Yes - reset
                progress.max_unlocked_level = 0;
                save_progress(&progress);
            }
            for entity in &overlay_query {
                commands.entity(entity).despawn();
            }
            confirming.0 = false;
        }
        return;
    }

    // Normal settings input
    if keyboard.just_pressed(KeyCode::Escape) {
        go_back(&origin, &mut next_state);
        return;
    }

    if keyboard.just_pressed(KeyCode::Enter) {
        match selected.0 {
            4 => {
                changed.0 = false;
                return;
            }
            5 => {
                confirming.0 = true;
                spawn_reset_overlay(&mut commands, &mut selected_confirm);
                return;
            }
            6 => {
                go_back(&origin, &mut next_state);
                return;
            }
            _ => {}
        }
    }

    let left = keyboard.just_pressed(KeyCode::ArrowLeft);
    let right = keyboard.just_pressed(KeyCode::ArrowRight);

    if !left && !right {
        return;
    }

    match selected.0 {
        0 => {
            if right {
                settings.music_volume = (settings.music_volume + 0.1).min(1.0);
            } else {
                settings.music_volume = (settings.music_volume - 0.1).max(0.0);
            }
            changed.0 = true;
        }
        1 => {
            if right {
                settings.sfx_volume = (settings.sfx_volume + 0.1).min(1.0);
            } else {
                settings.sfx_volume = (settings.sfx_volume - 0.1).max(0.0);
            }
            changed.0 = true;
        }
        2 => {
            settings.resolution = if settings.resolution == (1280, 720) {
                (1920, 1080)
            } else {
                (1280, 720)
            };
            if let Ok(mut window) = windows.single_mut() {
                window.resolution.set(settings.resolution.0 as f32, settings.resolution.1 as f32);
            }
            changed.0 = true;
        }
        3 => {
            settings.fullscreen = !settings.fullscreen;
            if let Ok(mut window) = windows.single_mut() {
                window.mode = if settings.fullscreen {
                    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                } else {
                    WindowMode::Windowed
                };
            }
            changed.0 = true;
        }
        _ => {}
    }
}
```

**Step 5: Update `settings_highlight` for popup buttons**

Add `selected_confirm: Res<SelectedConfirmItem>` and `confirming: Res<ConfirmingReset>` parameters (confirming is already there). Highlight popup buttons (rows 100, 101) when confirming:

```rust
fn settings_highlight(
    selected: Res<SelectedSettingsItem>,
    changed: Res<SettingsChanged>,
    confirming: Res<ConfirmingReset>,
    selected_confirm: Res<SelectedConfirmItem>,
    mut rows: Query<(&SettingsRow, &mut BackgroundColor)>,
) {
    for (row, mut bg) in &mut rows {
        let is_selected = row.0 == selected.0;

        // Popup buttons (100 = Yes, 101 = No)
        if row.0 >= 100 {
            let confirm_index = row.0 - 100;
            if confirming.0 && confirm_index == selected_confirm.0 {
                if confirm_index == 0 {
                    // Yes button highlighted red
                    *bg = BackgroundColor(Color::srgb(0.7, 0.15, 0.15));
                } else {
                    // No button highlighted blue
                    *bg = BackgroundColor(COLOR_SELECTED);
                }
            } else {
                *bg = BackgroundColor(COLOR_NORMAL);
            }
            continue;
        }

        // Normal settings rows
        if row.0 == 4 && changed.0 {
            if is_selected {
                *bg = BackgroundColor(Color::srgb(0.2, 0.8, 0.2));
            } else {
                *bg = BackgroundColor(Color::srgb(0.1, 0.4, 0.1));
            }
        } else if is_selected {
            *bg = BackgroundColor(COLOR_SELECTED);
        } else {
            *bg = BackgroundColor(COLOR_NORMAL);
        }
    }
}
```

**Step 6: Block settings navigation while popup is open**

In `settings_navigation`, add `confirming: Res<ConfirmingReset>` parameter and return early if popup is open:

```rust
fn settings_navigation(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedSettingsItem>,
    confirming: Res<ConfirmingReset>,
) {
    if confirming.0 {
        return;
    }
    // ... rest unchanged
}
```

**Step 7: Use `despawn_recursive()` instead of `despawn()`**

When closing the overlay, use `commands.entity(entity).despawn_recursive()` to ensure all children are also removed (the overlay has nested children for the panel, text, and buttons).

Replace both instances of `commands.entity(entity).despawn()` with `commands.entity(entity).despawn_recursive()`.

**Step 8: Verify**

Run: `cargo check`

**Step 9: Commit**

```bash
git add src/settings.rs
git commit -m "feat: replace inline reset confirmation with overlay popup"
```

---

## Task 4: Clippy and final verification

**Files:**
- Modify: various (as needed)

**Step 1: Run clippy**

Run: `cargo clippy -- -W clippy::all`
Fix any warnings.

**Step 2: Full game flow test**

Verify:
1. Menu → Play → Level Select → Level 1 → complete → auto-advances to Level 2 (NOT menu)
2. Only level 2 unlocks (not all levels)
3. Complete level 2 → auto-advances to Level 3
4. Player movement on platforms is smooth (no jerky/stuttering)
5. Complete all 5 levels → returns to Level Select
6. Settings → Reset Progress → overlay popup appears with Yes/No buttons
7. Arrow keys navigate between Yes/No in the popup
8. "No" or Esc closes popup without resetting
9. "Yes" resets progress, only level 1 unlocked afterwards
10. Settings navigation is blocked while popup is open

**Step 3: Commit (if changes were made)**

```bash
git add -A
git commit -m "chore: clippy fixes and final polish round 2"
```

---

## Summary

| # | Task | Key files | Type |
|---|------|-----------|------|
| 1 | Fix level transitions + unlock bug | states.rs, level.rs | Bug fix |
| 2 | Fix platform friction | level.rs | Bug fix |
| 3 | Reset Progress overlay popup | settings.rs | Improvement |
| 4 | Clippy & verify | various | Polish |

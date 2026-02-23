# SimplePlatformer Polish Round 3 — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Debug and fix jerky platform movement, fix reset popup visuals, add mouse support to all menus.

**Architecture:** Debug movement by adding temporary logging in Update and FixedUpdate, analyze output, then apply fix (likely moving player systems to FixedUpdate). Fix reset popup by adding background and padding to the panel. Add mouse support by reading Bevy's built-in `Interaction` component on `Button` entities in all screen modules.

**Tech Stack:** Rust, Bevy 0.18, avian2d 0.5

**Design doc:** `docs/plans/2026-02-23-polish3-design.md`

---

## Task 1: Debug jerky movement — add temporary logging

**Files:**
- Modify: `src/player.rs`

**Step 1: Add debug logging to `player_movement`**

Change `player_movement` (line 75–95) to log velocity before/after and position when the player is on the ground and moving:

```rust
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut LinearVelocity, &Grounded, &Transform), With<Player>>,
) {
    for (mut velocity, grounded, transform) in &mut query {
        let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
        let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);

        let vel_before = velocity.0;

        if left && !right {
            velocity.x = -300.0;
        } else if right && !left {
            velocity.x = 300.0;
        } else {
            velocity.x = 0.0;
        }

        // DEBUG
        if grounded.0 && (left || right) {
            info!(
                "[UPDATE move] before=({:.1},{:.1}) after=({:.1},{:.1}) pos=({:.1},{:.1})",
                vel_before.x, vel_before.y,
                velocity.x, velocity.y,
                transform.translation.x, transform.translation.y,
            );
        }

        if keyboard.just_pressed(KeyCode::Space) && grounded.0 {
            velocity.y = 500.0;
        }
    }
}
```

**Step 2: Add FixedUpdate debug system**

Register a new system in `PlayerPlugin::build`:
```rust
.add_systems(
    FixedUpdate,
    debug_physics_velocity.run_if(in_state(GameState::Playing)),
)
```

Add the function:
```rust
fn debug_physics_velocity(
    query: Query<(&LinearVelocity, &Grounded, &Transform), With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);
    if !left && !right {
        return;
    }
    for (velocity, grounded, transform) in &query {
        if grounded.0 {
            info!(
                "[FIXED phys]  vel=({:.1},{:.1}) pos=({:.1},{:.1})",
                velocity.x, velocity.y,
                transform.translation.x, transform.translation.y,
            );
        }
    }
}
```

**Step 3: Verify**

Run: `cargo check`

**Step 4: Test**

Run: `cargo run`
Walk on a platform (hold D key) for 2-3 seconds. Stop the game (Ctrl+C). Copy the console output.

**What to look for:**
- If `vel_before.x` shows values other than ±300.0 (e.g. 0.0 or small values), the physics solver is resetting our velocity between frames.
- If `[FIXED phys]` shows different velocity than what `[UPDATE move]` sets, there's a desync.
- If grounded flickers in the logs, the ground sensor is unstable.

**Step 5: DO NOT commit** — these are temporary debug logs.

---

## Task 2: Fix jerky movement based on debug results

**Files:**
- Modify: `src/player.rs`

**Most likely scenario:** The physics solver (FixedUpdate) overwrites the velocity we set in Update, or there's a timing mismatch.

**Step 1: Move `ground_detection` and `player_movement` from `Update` to `FixedUpdate`**

In `PlayerPlugin::build`, change:
```rust
// OLD (Update):
.add_systems(
    Update,
    (ground_detection, player_movement)
        .chain()
        .run_if(in_state(GameState::Playing)),
)
```
to:
```rust
// NEW (FixedUpdate):
.add_systems(
    FixedUpdate,
    (ground_detection, player_movement)
        .chain()
        .run_if(in_state(GameState::Playing)),
)
```

This ensures player input processing is synchronized with the physics solver.

**Step 2: Remove all debug logging**

- Remove the `debug_physics_velocity` system and its registration
- Revert `player_movement` back to its original signature (`Query<(&mut LinearVelocity, &Grounded), With<Player>>`) — remove `Transform` from the query, remove the `vel_before` variable, remove the `info!()` call

The clean `player_movement` should be:
```rust
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut LinearVelocity, &Grounded), With<Player>>,
) {
    for (mut velocity, grounded) in &mut query {
        let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
        let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);

        if left && !right {
            velocity.x = -300.0;
        } else if right && !left {
            velocity.x = 300.0;
        } else {
            velocity.x = 0.0;
        }

        if keyboard.just_pressed(KeyCode::Space) && grounded.0 {
            velocity.y = 500.0;
        }
    }
}
```

**Step 3: Verify**

Run: `cargo run`
Expected: smooth movement on platforms, no stuttering.

**Step 4: Commit**

```bash
git add src/player.rs
git commit -m "fix: move player systems to FixedUpdate to fix jerky platform movement"
```

**Note:** If FixedUpdate doesn't fix it and debug shows a different root cause, adjust the fix accordingly. The debug output from Task 1 will tell us exactly what's happening.

---

## Task 3: Fix reset progress popup visuals

**Files:**
- Modify: `src/settings.rs`

The popup panel (line 208–214) has no `BackgroundColor` or `padding`, so text floats transparently over the settings. Additionally `despawn()` should be `despawn_recursive()` to properly clean up children.

**Step 1: Add background and padding to the panel Node**

In `spawn_reset_overlay`, change the inner panel spawn (line 208–214) from:
```rust
overlay
    .spawn(Node {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    })
```
to:
```rust
overlay
    .spawn((
        Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        BorderRadius::all(Val::Px(12.0)),
    ))
```

Note the addition of:
- `padding: UiRect::all(Val::Px(40.0))` — space around content
- `BackgroundColor(Color::srgb(0.12, 0.12, 0.12))` — dark opaque background
- `BorderRadius::all(Val::Px(12.0))` — rounded corners
- The `.spawn(Node { ... })` becomes `.spawn((Node { ... }, BackgroundColor(...), BorderRadius(...)))` — tuple instead of single component

**Step 2: Fix `despawn()` → `despawn_recursive()`**

In `settings_adjust`, there are two places where the overlay is despawned (lines 280 and 297):
```rust
commands.entity(entity).despawn();
```

Change both to:
```rust
commands.entity(entity).despawn_recursive();
```

This ensures all child entities (panel, text, buttons) are properly cleaned up.

**Step 3: Verify**

Run: `cargo run`
Go to Settings → Reset Progress → overlay should now have a dark background panel with visible text and Yes/No buttons.

**Step 4: Commit**

```bash
git add src/settings.rs
git commit -m "fix: add background to reset popup panel and fix entity cleanup"
```

---

## Task 4: Add mouse support to main menu

**Files:**
- Modify: `src/menu.rs`

Bevy's `Button` component automatically tracks `Interaction` (None → Hovered → Pressed). We need a system that:
- On hover: syncs `SelectedMenuItem` to the hovered button's index
- On press: triggers the same action as Enter

**Step 1: Add `menu_mouse` system**

Add to the system chain in `MenuPlugin::build`. Insert it before `menu_highlight`:

```rust
(menu_navigation, menu_mouse, menu_highlight, menu_action)
```

The system:
```rust
fn menu_mouse(
    mut selected: ResMut<SelectedMenuItem>,
    buttons: Query<(&MenuAction, &Interaction), Changed<Interaction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut simulated_enter: Local<bool>,
) {
    let actions_order = [MenuAction::StartGame, MenuAction::Settings, MenuAction::Exit];

    for (action, interaction) in &buttons {
        let index = actions_order.iter().position(|a| a == action).unwrap_or(0);
        match interaction {
            Interaction::Hovered => {
                selected.0 = index;
            }
            Interaction::Pressed => {
                selected.0 = index;
            }
            Interaction::None => {}
        }
    }
}
```

**Step 2: Handle mouse click as action trigger**

The simplest approach: instead of checking only `keyboard.just_pressed(KeyCode::Enter)` in `menu_action`, also check for `Interaction::Pressed` on the selected button.

Change `menu_action` to also accept clicks. Add a parameter `buttons: Query<(&MenuAction, &Interaction)>`:

```rust
fn menu_action(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedMenuItem>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings_origin: ResMut<SettingsOrigin>,
    mut exit_events: MessageWriter<AppExit>,
    buttons: Query<(&MenuAction, &Interaction)>,
) {
    let actions_order = [MenuAction::StartGame, MenuAction::Settings, MenuAction::Exit];

    let mut clicked = false;
    for (action, interaction) in &buttons {
        if *interaction == Interaction::Pressed {
            let index = actions_order.iter().position(|a| a == action).unwrap_or(0);
            selected.into_inner().0 = index;
            clicked = true;
        }
    }

    if !keyboard.just_pressed(KeyCode::Enter) && !clicked {
        return;
    }

    // rest unchanged...
}
```

Wait — `selected` is `Res`, not `ResMut`. The mouse system (`menu_mouse`) already updates `selected` on Pressed. Since the systems are chained (`menu_mouse` before `menu_action`), by the time `menu_action` runs, `selected` is already correct.

So in `menu_action`, just add a click check:

```rust
fn menu_action(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedMenuItem>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings_origin: ResMut<SettingsOrigin>,
    mut exit_events: MessageWriter<AppExit>,
    buttons: Query<(&MenuAction, &Interaction)>,
) {
    let enter = keyboard.just_pressed(KeyCode::Enter);
    let clicked = buttons.iter().any(|(_, i)| *i == Interaction::Pressed);

    if !enter && !clicked {
        return;
    }

    match selected.0 {
        0 => {
            next_state.set(GameState::LevelSelect);
        }
        1 => {
            *settings_origin = SettingsOrigin::Menu;
            next_state.set(GameState::Settings);
        }
        2 => {
            exit_events.write(AppExit::default());
        }
        _ => {}
    }
}
```

**Step 3: Verify**

Run: `cargo run`
Expected: hovering over menu buttons highlights them, clicking activates them.

**Step 4: Commit**

```bash
git add src/menu.rs
git commit -m "feat: add mouse hover and click support to main menu"
```

---

## Task 5: Add mouse support to pause menu

**Files:**
- Modify: `src/pause.rs`

Same pattern as Task 4.

**Step 1: Add `pause_mouse` system**

In `PausePlugin::build`, update the system chain:
```rust
(pause_navigation, pause_mouse, pause_highlight, pause_action)
```

```rust
fn pause_mouse(
    mut selected: ResMut<SelectedPauseItem>,
    buttons: Query<(&PauseAction, &Interaction), Changed<Interaction>>,
) {
    let actions_order = [PauseAction::Resume, PauseAction::Settings, PauseAction::ToMenu];

    for (action, interaction) in &buttons {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            let index = actions_order.iter().position(|a| a == action).unwrap_or(0);
            selected.0 = index;
        }
    }
}
```

**Step 2: Update `pause_action` to handle clicks**

```rust
fn pause_action(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedPauseItem>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings_origin: ResMut<SettingsOrigin>,
    buttons: Query<(&PauseAction, &Interaction)>,
) {
    let enter = keyboard.just_pressed(KeyCode::Enter);
    let clicked = buttons.iter().any(|(_, i)| *i == Interaction::Pressed);

    if !enter && !clicked {
        return;
    }

    match selected.0 {
        0 => {
            next_state.set(GameState::Playing);
        }
        1 => {
            *settings_origin = SettingsOrigin::Paused;
            next_state.set(GameState::Settings);
        }
        2 => {
            next_state.set(GameState::Menu);
        }
        _ => {}
    }
}
```

**Step 3: Verify**

Run: `cargo check`

**Step 4: Commit**

```bash
git add src/pause.rs
git commit -m "feat: add mouse hover and click support to pause menu"
```

---

## Task 6: Add mouse support to level select

**Files:**
- Modify: `src/level_select.rs`

**Step 1: Add `level_select_mouse` system**

In `LevelSelectPlugin::build`, update the chain:
```rust
(level_select_navigation, level_select_mouse, level_select_highlight, level_select_action)
```

```rust
fn level_select_mouse(
    mut selected: ResMut<SelectedLevelItem>,
    buttons: Query<(&LevelSelectRow, &Interaction), Changed<Interaction>>,
) {
    for (row, interaction) in &buttons {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            selected.0 = row.0;
        }
    }
}
```

**Step 2: Update `level_select_action` to handle clicks**

```rust
fn level_select_action(
    keyboard: Res<ButtonInput<KeyCode>>,
    selected: Res<SelectedLevelItem>,
    progress: Res<PlayerProgress>,
    mut current_level: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<GameState>>,
    buttons: Query<(&LevelSelectRow, &Interaction)>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
        return;
    }

    let enter = keyboard.just_pressed(KeyCode::Enter);
    let clicked = buttons.iter().any(|(_, i)| *i == Interaction::Pressed);

    if !enter && !clicked {
        return;
    }

    if selected.0 < LEVELS.len() {
        if selected.0 <= progress.max_unlocked_level {
            current_level.0 = selected.0;
            next_state.set(GameState::Playing);
        }
    } else {
        next_state.set(GameState::Menu);
    }
}
```

**Step 3: Verify**

Run: `cargo check`

**Step 4: Commit**

```bash
git add src/level_select.rs
git commit -m "feat: add mouse hover and click support to level select"
```

---

## Task 7: Add mouse support to settings and reset popup

**Files:**
- Modify: `src/settings.rs`

Settings is more complex because it has:
- Regular settings rows (0–6) with Left/Right arrow adjustments
- Reset popup with Yes/No buttons (rows 100, 101)

**Step 1: Add `settings_mouse` system**

In `SettingsPlugin::build`, update the chain:
```rust
(settings_navigation, settings_mouse, settings_adjust, settings_highlight, settings_update_text)
```

```rust
fn settings_mouse(
    mut selected: ResMut<SelectedSettingsItem>,
    confirming: Res<ConfirmingReset>,
    mut selected_confirm: ResMut<SelectedConfirmItem>,
    buttons: Query<(&SettingsRow, &Interaction), Changed<Interaction>>,
) {
    for (row, interaction) in &buttons {
        if *interaction == Interaction::Hovered || *interaction == Interaction::Pressed {
            if confirming.0 && row.0 >= 100 {
                // Popup buttons
                selected_confirm.0 = row.0 - 100;
            } else if !confirming.0 && row.0 < 100 {
                // Normal settings rows
                selected.0 = row.0;
            }
        }
    }
}
```

**Step 2: Update `settings_adjust` to handle mouse clicks**

In `settings_adjust`, after the confirming block that handles keyboard, add click detection for popup buttons:

For the **popup** (when `confirming.0`), after the keyboard handling for Enter:
```rust
// Mouse click on popup buttons
let popup_clicked = overlay_query.iter().next().is_some()
    && buttons_interaction.iter().any(|(row, i)| row.0 >= 100 && *i == Interaction::Pressed);
```

Actually, the simplest approach: add a `buttons_interaction: Query<(&SettingsRow, &Interaction)>` parameter to `settings_adjust` and check for Pressed:

In the **confirming block**, after the Enter check, add:
```rust
// Mouse click
for (row, interaction) in &buttons_interaction {
    if *interaction == Interaction::Pressed && row.0 >= 100 {
        let confirm_index = row.0 - 100;
        if confirm_index == 0 {
            // Yes
            progress.max_unlocked_level = 0;
            save_progress(&progress);
        }
        for entity in &overlay_query {
            commands.entity(entity).despawn_recursive();
        }
        confirming.0 = false;
        return;
    }
}
```

In the **normal block**, after the Enter match, add:
```rust
// Mouse click on settings rows
for (row, interaction) in &buttons_interaction {
    if *interaction == Interaction::Pressed && row.0 < 100 {
        match row.0 {
            4 => { changed.0 = false; return; }
            5 => {
                confirming.0 = true;
                spawn_reset_overlay(&mut commands, &mut selected_confirm);
                return;
            }
            6 => { go_back(&origin, &mut next_state); return; }
            _ => {}
        }
    }
}
```

Add `buttons_interaction: Query<(&SettingsRow, &Interaction)>` as a new parameter.

**Step 3: Verify**

Run: `cargo run`
Expected: hovering over settings rows highlights them. Clicking on Save/Reset Progress/Back works. In the popup, hovering over Yes/No buttons highlights them, clicking works.

**Step 4: Commit**

```bash
git add src/settings.rs
git commit -m "feat: add mouse hover and click support to settings and reset popup"
```

---

## Task 8: Clippy and final verification

**Files:**
- Modify: various (as needed)

**Step 1: Run clippy**

Run: `cargo clippy -- -W clippy::all`
Fix any warnings.

**Step 2: Full game flow test**

Verify:
1. Movement on platforms is smooth (no jerk)
2. Reset Progress popup has dark background panel, Yes/No text visible
3. Mouse hover highlights buttons in: Menu, Pause, Level Select, Settings
4. Mouse click activates buttons in: Menu, Pause, Level Select, Settings
5. Mouse click on Yes/No in reset popup works
6. Keyboard navigation still works in all menus
7. Arrow keys for volume/resolution/fullscreen still work in settings

**Step 3: Commit (if changes)**

```bash
git add -A
git commit -m "chore: clippy fixes and final polish round 3"
```

---

## Summary

| # | Task | Key files | Type |
|---|------|-----------|------|
| 1 | Debug jerky movement (add logs) | player.rs | Debug |
| 2 | Fix jerky movement (FixedUpdate) | player.rs | Bug fix |
| 3 | Fix reset popup visuals | settings.rs | Bug fix |
| 4 | Mouse support: main menu | menu.rs | Feature |
| 5 | Mouse support: pause menu | pause.rs | Feature |
| 6 | Mouse support: level select | level_select.rs | Feature |
| 7 | Mouse support: settings + popup | settings.rs | Feature |
| 8 | Clippy & verify | various | Polish |

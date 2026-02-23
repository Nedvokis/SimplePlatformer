# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SimplePlatformer — 2D side-scrolling platformer built with Rust and Bevy 0.18. Player jumps across platforms to reach the level exit. Uses Bevy's `2d` feature set (default features disabled).

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

Key Bevy patterns to follow:
- Use `States` for game states (menu, gameplay, pause, settings)
- Use `Component` derive for entity data, `Resource` for global data
- Use `SystemSet` for ordering related systems
- Assets go in `assets/` (Bevy's default asset path)

## Game Design

- **Main menu** at launch: Start Game / Settings / Exit
- **Controls**: A/← left, D/→ right, Space jump, Esc menu
- **Window**: 1280×720, title "SimplePlatformer"

## Conventions

- Language in code: English
- Language in UI/docs: Russian
- Bevy 0.18 API — `WindowResolution` takes `u32` tuples, not floats
- Dev profile: `opt-level = 1` for game code, `opt-level = 3` for dependencies (faster iteration)

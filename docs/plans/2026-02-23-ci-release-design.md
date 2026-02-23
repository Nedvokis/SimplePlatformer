# CI Release Builds — Design

## Goal

Automatically build game binaries for Linux, Windows, and macOS when a version tag is pushed, and attach them to a GitHub Release.

## Trigger

Push of a tag matching `v*` (e.g. `v0.1.0`).

## Architecture

GitHub Actions workflow with two jobs:

1. **`build`** — matrix strategy, 3 parallel runners:
   - `ubuntu-latest` → Linux x86_64
   - `windows-latest` → Windows x86_64
   - `macos-latest` → macOS x86_64

   Each runner:
   1. Checkout repo
   2. Install Rust stable
   3. Install system dependencies (Linux only: libasound2-dev, libudev-dev, libxkbcommon-dev, libwayland-dev)
   4. `cargo build --release`
   5. Package binary + `assets/` into ZIP
   6. Upload as workflow artifact

2. **`release`** — runs after all builds complete:
   1. Download all 3 ZIP artifacts
   2. Create GitHub Release from tag
   3. Attach all 3 ZIPs to the release

## Naming

Archive: `SimplePlatformer-{tag}-{os}-x86_64.zip`

Example: `SimplePlatformer-v0.1.0-linux-x86_64.zip`

## ZIP Contents

```
SimplePlatformer-v0.1.0-linux-x86_64/
├── simple_platformer        (or .exe on Windows)
└── assets/
    └── levels/
        ├── level_01.ron
        ├── level_02.ron
        ├── level_03.ron
        ├── level_04.ron
        └── level_05.ron
```

## File

`.github/workflows/release.yml`

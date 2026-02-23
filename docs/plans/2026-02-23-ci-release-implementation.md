# CI Release Builds — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up GitHub Actions to build game binaries for Linux, Windows, and macOS on tag push, and attach them to a GitHub Release.

**Architecture:** Single workflow file with a `build` matrix job (3 OS runners in parallel) that compiles and packages, then a `release` job that creates a GitHub Release and attaches all ZIP artifacts.

**Tech Stack:** GitHub Actions, Rust stable, cargo, zip

**Design doc:** `docs/plans/2026-02-23-ci-release-design.md`

---

## Task 1: Create the release workflow

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Create directory structure**

```bash
mkdir -p .github/workflows
```

**Step 2: Write the workflow file**

Create `.github/workflows/release.yml` with the following content:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always
  BINARY_NAME: simple_platformer

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact_name: linux-x86_64
            binary_ext: ""
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact_name: windows-x86_64
            binary_ext: ".exe"
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact_name: macos-x86_64
            binary_ext: ""

    runs-on: ${{ matrix.os }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install Linux dependencies
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libasound2-dev libudev-dev libxkbcommon-dev libwayland-dev

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package (Linux/macOS)
        if: matrix.os != 'windows-latest'
        run: |
          TAG=${GITHUB_REF#refs/tags/}
          DIR_NAME="SimplePlatformer-${TAG}-${{ matrix.artifact_name }}"
          mkdir -p "${DIR_NAME}"
          cp "target/${{ matrix.target }}/release/${BINARY_NAME}" "${DIR_NAME}/"
          cp -r assets "${DIR_NAME}/"
          zip -r "${DIR_NAME}.zip" "${DIR_NAME}"

      - name: Package (Windows)
        if: matrix.os == 'windows-latest'
        shell: pwsh
        run: |
          $tag = $env:GITHUB_REF -replace 'refs/tags/', ''
          $dirName = "SimplePlatformer-${tag}-${{ matrix.artifact_name }}"
          New-Item -ItemType Directory -Path $dirName
          Copy-Item "target/${{ matrix.target }}/release/${env:BINARY_NAME}.exe" $dirName
          Copy-Item -Recurse assets $dirName
          Compress-Archive -Path $dirName -DestinationPath "${dirName}.zip"

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: SimplePlatformer-${{ matrix.artifact_name }}
          path: SimplePlatformer-*.zip
          retention-days: 1

  release:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write

    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          merge-multiple: true

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: SimplePlatformer-*.zip
          generate_release_notes: true
```

**Step 3: Verify YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))" 2>&1 || echo "If python yaml not available, just visually verify indentation"`

Alternatively, just verify the file looks correct with `cat .github/workflows/release.yml`.

**Step 4: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add GitHub Actions release workflow for cross-platform builds

Builds game binaries for Linux, Windows, and macOS when a version tag
is pushed. Packages each binary with assets/ into a ZIP and attaches
them to a GitHub Release.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 2: Test the workflow

This task is manual — you need to push a tag to trigger the workflow.

**Step 1: Push current commits to origin**

```bash
git push origin master
```

**Step 2: Create and push a test tag**

```bash
git tag v0.1.0
git push origin v0.1.0
```

**Step 3: Verify**

Go to `https://github.com/Nedvokis/SimplePlatformer/actions` and watch the workflow run.

Expected:
- 3 build jobs run in parallel (Linux, Windows, macOS)
- After all 3 complete, release job creates a GitHub Release
- Release page at `https://github.com/Nedvokis/SimplePlatformer/releases` has 3 ZIP files attached

**Step 4: If build fails**

Check the logs for the failing job. Common issues:
- **Linux**: missing system dependencies → add to apt-get install
- **Windows**: path separators → use pwsh
- **macOS**: missing SDK → may need to add Xcode setup step

---

## Summary

| # | Task | Key files | Type |
|---|------|-----------|------|
| 1 | Create release workflow | .github/workflows/release.yml | Feature |
| 2 | Test the workflow (manual) | — | Verification |

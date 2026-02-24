# SimplePlatformer build commands

# Extract version from Cargo.toml
version := `grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/'`

# Run the game in debug mode
run:
    cargo run

# Build release binary
build:
    cargo build --release

# Fast compilation check
check:
    cargo check

# Run linter
lint:
    cargo clippy

# Run all tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Build and package for Linux and Windows
package: _package-linux _package-windows
    @echo "Packages created in dist/"
    @ls -la dist/*.zip

# Package Linux build
_package-linux:
    cargo build --release --target x86_64-unknown-linux-gnu
    @mkdir -p dist/SimplePlatformer-v{{version}}-linux-x86_64
    cp target/x86_64-unknown-linux-gnu/release/simple_platformer dist/SimplePlatformer-v{{version}}-linux-x86_64/
    cp -r assets dist/SimplePlatformer-v{{version}}-linux-x86_64/
    cd dist && zip -r SimplePlatformer-v{{version}}-linux-x86_64.zip SimplePlatformer-v{{version}}-linux-x86_64/
    rm -rf dist/SimplePlatformer-v{{version}}-linux-x86_64/

# Package Windows build (requires mingw-w64)
_package-windows:
    cargo build --release --target x86_64-pc-windows-gnu
    @mkdir -p dist/SimplePlatformer-v{{version}}-windows-x86_64
    cp target/x86_64-pc-windows-gnu/release/simple_platformer.exe dist/SimplePlatformer-v{{version}}-windows-x86_64/
    cp -r assets dist/SimplePlatformer-v{{version}}-windows-x86_64/
    cd dist && zip -r SimplePlatformer-v{{version}}-windows-x86_64.zip SimplePlatformer-v{{version}}-windows-x86_64/
    rm -rf dist/SimplePlatformer-v{{version}}-windows-x86_64/

#!/bin/bash

set -eu

# Preflight checks: ensure Docker daemon and `cross` are available before starting
if ! command -v docker >/dev/null 2>&1; then
    echo "Docker is not installed or not in PATH. Please install Docker and try again."
    exit 1
fi

if ! docker info >/dev/null 2>&1; then
    echo "Docker daemon is not running or /var/run/docker.sock is not accessible. Start Docker and retry."
    exit 1
fi

if ! command -v cross >/dev/null 2>&1; then
    echo "'cross' is not installed or not in PATH. Install it with: cargo install cross"
    exit 1
fi

# List of target platforms
TARGETS=( \
x86_64-unknown-linux-gnu        \
x86_64-pc-windows-gnu           \
aarch64-unknown-linux-gnu       \
armv7-unknown-linux-gnueabihf   \
arm-unknown-linux-gnueabihf     \
)

rm -rf target/assets
mkdir -p target/assets 2>/dev/null || true

# Cross-compile for each target platform
for target in "${TARGETS[@]}"; do
    echo; echo "########################################################################################";
    echo "Cross-compiling gui version for $target..."; echo
    cross build --release --target "$target"
done

# Test natively
cargo build --release
cargo test --release

# Copy the compiled files
for target in "${TARGETS[@]}"; do
    ext=""
    if [ $target = "x86_64-pc-windows-gnu" ]; then
        ext=".exe"
    fi

    cp -vf target/${target}/release/quick-serve-gui${ext}   target/assets/quick-serve-gui-${target}${ext} || true
    cp -vf target/${target}/release/quick-serve${ext}       target/assets/quick-serve-${target}${ext} || true
done


if [[ "$*" == *--release* ]]; then
    # Tag the new version
    version=$(grep '^version =' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
    notes=$(git log --pretty=format:'- %s' $(git describe --tags --abbrev=0 @^)..@)
    # notes=$(git log $(git describe --tags --abbrev=0)..HEAD --oneline)

    echo "Creating release v$version with notes:"
    echo "$notes"

    echo "Tagging the new version..."
    git tag "v$version" -f
    git push origin --tags -f

    echo "Creating a release on GitHub..."
    gh release delete "v$version" || true
    # Create a release on GitHub and upload the files
    gh release create "v$version" target/assets/* --title "v$version" --notes "$notes" --generate-notes

    # Publish the new version to crates.io
    echo; echo "Publishing the new version to crates.io..."
    read -p "Do you want publish? (y/N): " answer
    if [[ "$answer" == "y" ]]; then
        cargo publish --allow-dirty --no-verify
    else
        cargo publish --allow-dirty --no-verify --dry-run
    fi
fi


exit 0

#!/bin/bash

set -eu

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
    echo; echo "########################################################################################"; echo
    echo "Cross-compiling gui version for $target..."

    cross build --release --target "$target"
done

# Test natively
cross test --release

# Copy the compiled files
for target in "${TARGETS[@]}"; do
    ext=""
    if [ $target = "x86_64-pc-windows-gnu" ]; then
        ext=".exe"
    fi

    cp -vf target/${target}/release/quick-serve-gui${ext}   target/assets/quick-serve-gui-${target}${ext} || true
    cp -vf target/${target}/release/quick-serve${ext}       target/assets/quick-serve-${target}${ext} || true
done

exit 0

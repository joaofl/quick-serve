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

# cargo install cargo-zigbuild
pip3 install ziglang
rm -rf assets
mkdir assets 2>/dev/null || true

# Cross-compile for each target platform
for target in "${TARGETS[@]}"; do
        rustup target add $target

        echo "Cross-compiling the headless version for $target..."
        cargo zigbuild --release --target "$target"  --no-default-features --jobs $(nproc)

        echo "Cross-compiling gui version for $target..."
        cargo zigbuild --release --target "$target" --jobs $(nproc)

        echo "Copying $target assets"

        if [ $target = "x86_64-pc-windows-gnu" ]; then
            ext=".exe"
        else
            ext=""
        fi

        cp -v target/${target}/release/quick-serve${ext}           assets/quick-serve-${target}${ext}          2>/dev/null
        cp -v target/${target}/release/quick-serve-headless${ext}  assets/quick-serve-headless-${target}${ext} 2>/dev/null
done

# echo "Cleaning up..."
# cargo clean

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
        ext=""
        if [ $target = "x86_64-pc-windows-gnu" ]; then
            ext=".exe"
        fi

        # rustup target add $target

        echo "Cross-compiling gui version for $target..."
        # cargo zigbuild --release --target "$target" --jobs $(nproc) --bin quick-serve
        cross build --release --target "$target" --jobs $(nproc) --bin quick-serve
        cp -vf target/${target}/release/quick-serve${ext}     assets/quick-serve-gui-${target}${ext} || true

        echo "Cross-compiling the headless version for $target..."
        # cargo zigbuild --release --target "$target"  --no-default-features --jobs $(nproc)
        cross build --release --target "$target" --no-default-features --jobs $(nproc) --bin quick-serve
        cp -vf target/${target}/release/quick-serve${ext}  assets/quick-serve-${target}${ext} || true
done

# echo "Cleaning up..."
# cargo clean

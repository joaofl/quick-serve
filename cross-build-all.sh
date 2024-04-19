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

        echo "Cross-compiling gui version for $target..."
        cargo zigbuild --release --target "$target" --jobs $(nproc)

        echo "Cross-compiling the headless version for $target..."
        cargo zigbuild --release --target "$target"  --no-default-features --jobs $(nproc)
done

for target in "${TARGETS[@]}"; do
        ext=""
        if [ $target = "x86_64-pc-windows-gnu" ]; then
            ext=".exe"
        fi

        echo "Copying $target assets"
        cp -v target/${target}/release/quick-serve-gui${ext}    assets/quick-serve-gui-${target}${ext}
        cp -v target/${target}/release/quick-serve${ext}        assets/quick-serve${target}${ext}
done

# echo "Cleaning up..."
# cargo clean

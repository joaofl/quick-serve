#!/bin/bash

# List of target platforms
TARGETS=("x86_64-unknown-linux-gnu" "x86_64-pc-windows-gnu" "arm-unknown-linux-gnueabi" "aarch64-unknown-linux-gnu")

mkdir assets

# Cross-compile for each target platform
for target in "${TARGETS[@]}"; do
    echo "Cross-compiling for $target..."
    cross build --release --target "$target" --jobs $(nproc)
    echo "$target compilation completed."

    cp -v target/${target}/release/quick-serve     assets/quick-serve-${target}     2>/dev/null
    cp -v target/${target}/release/quick-serve.exe assets/quick-serve-${target}.exe 2>/dev/null
done

# Clean up
echo "Cleaning up..."
cargo clean

#!/bin/bash

set -eu

# List of target platforms
TARGETS=("x86_64-unknown-linux-gnu" "x86_64-pc-windows-gnu" "arm-unknown-linux-gnueabi" "aarch64-unknown-linux-gnu")

mkdir assets 2>/dev/null || true

# Cross-compile for each target platform
for target in "${TARGETS[@]}"; do
    rustup target add $target
    
    echo "Cross-compiling for $target..."
    cargo build --release --target "$target" --jobs $(nproc)
    # echo "$target compilation completed."

    if [ $target = "x86_64-pc-windows-gnu" ]; then
        cp -v target/${target}/release/quick-serve.exe assets/quick-serve-${target}.exe 2>/dev/null
    else
        cp -v target/${target}/release/quick-serve     assets/quick-serve-${target}     2>/dev/null
    fi
done

# Clean up
echo "Cleaning up..."
cargo clean

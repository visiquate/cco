#!/bin/bash

# WASM Terminal Build Script
# Builds the WASM terminal module using cargo and wasm-bindgen

set -e

echo "Building WASM terminal..."

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check if wasm-bindgen-cli is installed
if ! command -v wasm-bindgen &> /dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli --force
fi

# Add wasm32 target if not present
echo "Ensuring wasm32-unknown-unknown target is installed..."
if ! cargo build --target wasm32-unknown-unknown --lib --release --dry-run 2>/dev/null; then
    echo "Installing wasm32-unknown-unknown target..."
    # Use rustup if available, otherwise skip (Homebrew cargo may have target built-in)
    if command -v rustup &> /dev/null; then
        rustup target add wasm32-unknown-unknown
    else
        echo "Note: rustup not found, assuming target is available in Homebrew Rust"
    fi
fi

# Build WASM module
echo "Compiling Rust code to WebAssembly..."
cargo build --target wasm32-unknown-unknown --lib --release

# Create pkg directory if it doesn't exist
mkdir -p pkg

# Generate JavaScript bindings
echo "Generating JavaScript bindings..."
wasm-bindgen \
    --target web \
    --out-dir pkg \
    --out-name cco_wasm \
    target/wasm32-unknown-unknown/release/cco_wasm.rlib 2>/dev/null || \
wasm-bindgen \
    --target web \
    --out-dir pkg \
    --out-name cco_wasm \
    target/wasm32-unknown-unknown/release/libcco_wasm.rlib 2>/dev/null || \
{
    echo "Error: Could not find compiled WASM library"
    echo "Looking for available files:"
    ls -la target/wasm32-unknown-unknown/release/ | grep -E "(cco|wasm)" || echo "No WASM files found"
    exit 1
}

# Optimize with wasm-opt if available
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM with wasm-opt..."
    wasm-opt -Oz \
        --converge \
        --strip \
        -o pkg/cco_wasm_bg_optimized.wasm \
        pkg/cco_wasm_bg.wasm
    mv pkg/cco_wasm_bg_optimized.wasm pkg/cco_wasm_bg.wasm
else
    echo "wasm-opt not found, skipping optimization (install with: npm install -g wasm-opt)"
fi

# Report size
echo ""
echo "Build complete!"
if [ -f pkg/cco_wasm_bg.wasm ]; then
    echo "WASM module size: $(du -h pkg/cco_wasm_bg.wasm | cut -f1)"
fi
if [ -f pkg/cco_wasm.js ]; then
    echo "JavaScript glue size: $(du -h pkg/cco_wasm.js | cut -f1)"
fi

# Copy to static directory
echo ""
echo "Copying to static directory..."
cp pkg/cco_wasm_bg.wasm static/
cp pkg/cco_wasm.js static/

echo ""
echo "WASM terminal build successful!"
echo "Files copied to static/:"
echo "  - static/cco_wasm.js"
echo "  - static/cco_wasm_bg.wasm"
echo ""
echo "Test at: http://localhost:8000/static/test-wasm-terminal.html"
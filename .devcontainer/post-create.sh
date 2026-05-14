#!/usr/bin/env bash
set -euo pipefail

echo "==> RuralPeds dev container initialized"

# Verify toolchain
rustc --version
cargo --version
uv --version

# Warm dependency cache if a Cargo.toml exists
if [ -f "Cargo.toml" ]; then
    cargo fetch || echo "WARNING: cargo fetch failed; network may be restricted."
fi

echo "==> Ready for development."

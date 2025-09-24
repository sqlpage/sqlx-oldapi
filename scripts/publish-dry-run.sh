#!/usr/bin/env bash
set -euo pipefail

# Ensure we are at repo root
cd "$(dirname "$0")/.."

if [ -f "/usr/local/cargo/env" ]; then
  . "/usr/local/cargo/env"
fi
if command -v rustup >/dev/null 2>&1; then
  rustup show >/dev/null 2>&1 || true
fi

echo "cargo version: $(cargo --version)"
echo "Running: cargo publish --workspace --allow-dirty --dry-run --no-verify"
cargo publish --workspace --allow-dirty --dry-run --no-verify
echo "Dry-run finished"

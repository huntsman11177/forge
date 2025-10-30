#!/usr/bin/env bash
set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
VERSION_FILE="$WORKSPACE_ROOT/version.txt"

if [[ ! -f "$VERSION_FILE" ]]; then
  echo "version.txt not found at $WORKSPACE_ROOT" >&2
  exit 1
fi

if ! command -v melos >/dev/null 2>&1; then
  echo "Installing Melos globally..."
  dart pub global activate melos
fi

echo "Running melos bootstrap..."
(cd "$WORKSPACE_ROOT" && melos bootstrap)

echo "Ensuring Flutter environment is ready..."
flutter --version >/dev/null
flutter doctor

echo "Ensuring Rust toolchain is installed..."
if ! command -v rustup >/dev/null 2>&1; then
  echo "rustup is required to install Rust toolchains." >&2
  exit 1
fi
rustup toolchain install stable

ENGINE_DIR="$WORKSPACE_ROOT/packages/forge_engine"
echo "Building Forge engine (release)..."
(cd "$ENGINE_DIR" && cargo build --release)

ENGINE_BINARY="$ENGINE_DIR/target/release/forge_engine_cli"
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
  ENGINE_BINARY+=".exe"
fi

echo "Forge engine built at: $ENGINE_BINARY"

echo "Setup complete."

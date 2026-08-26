#!/usr/bin/env bash

set -e
DIR=$(realpath "$0") && DIR=${DIR%/*}
cd "$DIR"

ensure_cmd() {
  for cmd in "$@"; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
      echo "=== Installing $cmd ==="
      cargo install "$cmd"
    fi
  done
}

ensure_cmd wasm-pack wasm-opt

DIST_DIR="$DIR/dist"
mkdir -p "$DIST_DIR"

for pkg_dir in url/*/; do
  if [ -d "$pkg_dir" ]; then
    pkg_name=$(basename "$pkg_dir")
    out_dir="$DIST_DIR/$pkg_name"
    echo "=== Building $pkg_name for Cloudflare Workers WASM ==="
    wasm-pack build "$pkg_dir" --target web --release --out-dir "$out_dir"
    for wasm_file in "$out_dir"/*.wasm; do
      if [ -f "$wasm_file" ]; then
        wasm-opt -Oz --strip-debug --strip-producers --all-features "$wasm_file" -o "$wasm_file"
      fi
    done
  fi
done

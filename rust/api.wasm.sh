#!/usr/bin/env bash

set -e
DIR=$(realpath "$0") && DIR=${DIR%/*}
cd "$DIR"

. ./env.sh

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

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

wasm-pack build api --target web --release --out-dir "$TMP_DIR"

cp "$TMP_DIR/webc_api_bg.wasm" "$DIST_DIR/api.wasm"

format_size() {
  local size=$1
  if [ $size -ge 1048576 ]; then
    awk "BEGIN {printf \"%.2f MB (%'d bytes)\", $size/1048576, $size}"
  elif [ $size -ge 1024 ]; then
    awk "BEGIN {printf \"%.2f KB (%'d bytes)\", $size/1024, $size}"
  else
    printf "%'d bytes" "$size"
  fi
}

opt_bytes=$(wc -c <"$DIST_DIR/api.wasm" | tr -d ' ')

echo "api.wasm size : $(format_size $opt_bytes)"

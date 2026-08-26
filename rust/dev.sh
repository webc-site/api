#!/usr/bin/env bash

set -e
DIR=$(realpath "$0") && DIR=${DIR%/*}
cd "$DIR"
. sh/pid.sh
. ./env.sh

ensure_cmd() {
  for cmd in "$@"; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
      echo "=== Installing $cmd ==="
      cargo install "$cmd"
    fi
  done
}

ensure_cmd watchexec

set -x

exec watchexec \
  --shell=none \
  --project-origin "$DIR" \
  -w "$DIR/srv" \
  -w "$DIR/dist/api.wasm" \
  --exts rs,toml,wasm \
  -r \
  -- cargo run -p webc_srv

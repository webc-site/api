#!/usr/bin/env bash

set -e
DIR=$(realpath "$0") && DIR=${DIR%/*}
cd "$DIR"
ensure_cmd() {
  for cmd in "$@"; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
      cargo install "$cmd" >/dev/null 2>&1
    fi
  done
}

ensure_cmd wasm-bindgen wasm-opt

TARGET_DIR="${CARGO_TARGET_DIR:-$DIR/target}"

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

measure_example() {
  local name=$1
  cargo build --target wasm32-unknown-unknown --package webc_lib --example "$name" --release >/dev/null 2>&1

  local wasm_src="$TARGET_DIR/wasm32-unknown-unknown/release/examples/$name.wasm"
  if [ ! -f "$wasm_src" ]; then
    wasm_src=$(find "$TARGET_DIR" -name "$name.wasm" 2>/dev/null | head -n 1)
  fi

  if [ ! -f "$wasm_src" ]; then
    echo "Error: $name.wasm not found in $TARGET_DIR"
    return 1
  fi

  local out_dir="$DIR/dist/examples/$name"
  rm -rf "$out_dir"
  mkdir -p "$out_dir"

  RUST_LOG=error wasm-bindgen "$wasm_src" --out-dir "$out_dir" --target web >/dev/null 2>&1

  local raw_wasm="$out_dir/${name}_bg.wasm"
  local opt_wasm="$out_dir/${name}_bg.opt.wasm"

  wasm-opt -Oz --strip-debug --strip-producers --all-features "$raw_wasm" -o "$opt_wasm"
  gzip -9 -c "$opt_wasm" >"$opt_wasm.gz"

  local opt_bytes=$(wc -c <"$opt_wasm" | tr -d ' ')
  local gz_bytes=$(wc -c <"$opt_wasm.gz" | tr -d ' ')

  echo "[$name]"
  echo "wasm-opt -Oz : $(format_size $opt_bytes)"
  echo "gzip -9      : $(format_size $gz_bytes)"
  echo ""
}

for example_file in lib/examples/*.rs; do
  if [ -f "$example_file" ]; then
    name=$(basename "$example_file" .rs)
    measure_example "$name"
  fi
done

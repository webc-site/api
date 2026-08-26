#!/usr/bin/env bash

set -e
DIR=$(realpath "$0") && DIR=${DIR%/*}
cd "$DIR"

if [ -d "$DIR/rust" ]; then
  RUST_DIR="$DIR/rust"
else
  RUST_DIR="$DIR"
fi

cd "$RUST_DIR"
. ./env.sh

# 1. 编译最新的 dist/api.wasm
./api.wasm.sh

# 2. 配置端口与退出清理
PORT=18080
export LISTEN_ADDR="127.0.0.1:$PORT"
export WASM_PATH="$RUST_DIR/dist/api.wasm"

cleanup() {
  if [ -n "$SRV_PID" ]; then
    kill "$SRV_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

# 3. 后台启动 srv 服务
cargo run -p webc_srv &
SRV_PID=$!

# 4. 等待端口监听就绪
wait_port() {
  for _ in {1..50}; do
    if nc -z 127.0.0.1 "$PORT" 2>/dev/null; then
      return 0
    fi
    sleep 0.2
  done
  echo "Error: Server failed to start on port $PORT"
  return 1
}

wait_port
set -x
curl -fsSL "http://127.0.0.1:$PORT/"

curl -fsSL "http://127.0.0.1:$PORT/test_api"

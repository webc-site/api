#!/usr/bin/env bash

set -e
DIR=$(realpath "$0") && DIR=${DIR%/*}
cd "$DIR"
. ../sh/pid.sh
. ../env.sh

set -x

COMPOSE_FILE="$DIR/docker/docker-compose.yml"

cleanup() {
  docker compose -f "$COMPOSE_FILE" down -v --remove-orphans >/dev/null 2>&1 || true
}

trap cleanup EXIT INT TERM

wait_port() {
  local port=$1
  for _ in {1..30}; do
    if nc -z 127.0.0.1 "$port" 2>/dev/null; then
      return 0
    fi
    sleep 0.5
  done
  return 1
}

wait_cluster_ok() {
  for _ in {1..30}; do
    if docker exec redis-cluster redis-cli -p 7000 cluster info 2>/dev/null | grep -q "cluster_state:ok" && \
       [ "$(docker exec redis-cluster redis-cli -p 7000 cluster nodes 2>/dev/null | grep -c 'connected')" -ge 3 ]; then
      return 0
    fi
    sleep 0.5
  done
  return 1
}

# 1. 验证 WASM 跨平台编译
cargo build --target wasm32-unknown-unknown -p kvrocks

# 2. 启动总的 docker-compose（包含 Standalone 6665, Sentinel 6667+26379, Cluster 7000-7002）
cleanup
docker compose -f "$COMPOSE_FILE" up -d

# 3. 等待所有服务就绪
wait_port 6665
wait_port 6667
wait_port 26379
wait_port 7000
docker exec redis-cluster sh -c "for p in 7000 7001 7002; do redis-cli -p \$p config set protected-mode no 2>/dev/null || true; done" || true
wait_cluster_ok
docker exec redis-cluster sh -c "for p in 7000 7001 7002; do redis-cli -p \$p config set protected-mode no 2>/dev/null || true; done" || true
sleep 2

# 4. 使用 cargo nextest 一次性跑完所有测试套件
cargo nextest run -p kvrocks --all-targets --test-threads 1

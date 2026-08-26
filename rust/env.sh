#!/usr/bin/env bash

ENV_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

if [ ! -d "$ENV_DIR/node_modules" ]; then
  (cd "$ENV_DIR" && bun i)
fi

. "$ENV_DIR/sh/env.sh"
if [ -f "$ENV_DIR/../conf/env.sh" ]; then
  . "$ENV_DIR/../conf/env.sh"
fi




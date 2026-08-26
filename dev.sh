#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
. ./sh/pid.sh
if [ ! -d "conf" ]; then
  eacho "miss conf dir"
  exit 1
fi
set -x

exec bun --watch --port 3003 src/srv.js

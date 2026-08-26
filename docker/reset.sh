#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -x

pkill -9 -f "surreal start" || true
./down.js
rm -rf data
./up.js

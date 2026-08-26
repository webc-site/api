#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR

. ./env.sh

cargo nextest run --workspace --all-features --no-capture --exclude kvrocks

./kvrocks/test.sh

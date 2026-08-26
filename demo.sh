#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
export NODE_ENV=test
set -x

./demo/test.js

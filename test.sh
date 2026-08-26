#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
export NODE_ENV=test
set -x

bun x oxfmt >/dev/null
bun x oxlint --fix >/dev/null
bun x knip

bun ./docker/sdb/reset.js
bun test --only-failures

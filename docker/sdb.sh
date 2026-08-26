#!/usr/bin/env bash

set -e
DIR=$(realpath $0) && DIR=${DIR%/*}
cd $DIR
set -a
. .env
set +a
set -x
exec surreal sql --endpoint http://127.0.0.1:$SDB_PORT --user root --pass $SDB_PASSWORD

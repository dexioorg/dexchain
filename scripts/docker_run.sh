#!/usr/bin/env bash

set -e

echo "*** Start DEX Chain ***"

cd $(dirname ${BASH_SOURCE[0]})/..

mkdir -p ./.local

docker-compose down --remove-orphans
docker-compose run --rm --service-ports dev $@
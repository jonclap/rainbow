#!/usr/bin/env bash

set -e -u -o pipefail

mkdir _build

cd $GITHUB_WORKSPACE

echo "Run: $*"

bash -c "$*"

cp target/release/rainbow _build/
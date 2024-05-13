#!/bin/bash

set -euxo pipefail

VERSION=$(cargo get package.version)

docker run -v $PWD:$PWD -w $PWD sean0x42/markdown-extract -r "${VERSION}" CHANGELOG.md | tee release.txt

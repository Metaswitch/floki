#!/bin/bash

set -euxo pipefail

VERSION=$(tomlq -r '.package.version' Cargo.toml)

docker run -v $PWD:$PWD -w $PWD sean0x42/markdown-extract -r "${VERSION}" CHANGELOG.md | tee release.txt

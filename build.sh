#!/bin/sh

set -ex

if [ $RUNNER = "ubuntu-24.04" ]
then
  OS_ID="linux"
elif [ $RUNNER = "macos-latest" ]
then
  OS_ID="osx"
fi

TAG=$(cargo get package.version)

LABEL=${TAG}-${OS_ID}

echo "Starting release build for ${LABEL}"

if [ ${OS_ID} = "linux" ]
then
  echo "Building linux binary"
  cargo build --release

  FLOKI_BINARY=$(find target/ -name "floki")
  echo "FLOKI_BINARY is $FLOKI_BINARY"
  FLOKI_DIR=$(dirname $FLOKI_BINARY)

  # Check that it's statically compiled!
  ldd $FLOKI_BINARY || true

  # Strip .gnu.hash since floki is statically compiled.
  # Required so that cargo-generate-rpm does not introduce a dependency on the RTLD.
  strip --remove-section=.gnu.hash $FLOKI_BINARY

  tar -cvzf floki-${LABEL}.tar.gz -C $FLOKI_DIR floki
  tar -tvzf floki-${LABEL}.tar.gz
else
  echo "Building release binary"
  cargo build --release
  zip -j floki-${LABEL}.zip target/release/floki
fi

echo "Release build complete"

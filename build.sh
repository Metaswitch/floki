#!/bin/sh

set -ex

if [ $OS_NAME = "ubuntu-24.04" ]
then
  OS_ID="linux"
elif [ $OS_NAME = "macos-latest" ]
then
  OS_ID="osx"
fi

TAG=$(cargo get package.version)

LABEL=${TAG}-${OS_ID}

echo "Starting release build for ${LABEL}"

if [ ${OS_ID} = "linux" ]
then
  echo "Building statically linked linux binary"
  docker build -f .devcontainer/Dockerfile.alpine -t flokirust .

  docker run --rm -v $(pwd):/src -w /src flokirust \
    sh -c 'cargo build --release && cp target/x86_64-unknown-linux-musl/release/floki .'
  sudo chown -R $(id -u):$(id -g) .

  # Check that it's statically compiled!
  ldd floki

  # Strip .gnu.hash since floki is statically compiled.
  # Required so that cargo-generate-rpm does not introduce a dependency on the RTLD.
  strip --remove-section=.gnu.hash floki

  tar -cvzf floki-${LABEL}.tar.gz floki
else
  echo "Building release binary"
  cargo build --release
  zip -j floki-${LABEL}.zip target/release/floki
fi

echo "Release build complete"

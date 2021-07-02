#!/bin/sh

set -ex

if [ $OS_NAME = "ubuntu-18.04" ]
then
  OS_ID="linux"
elif [ $OS_NAME = "macos-latest" ]
then
  OS_ID="osx"
fi

TAG=$(tomlq -r '.package.version' Cargo.toml)

LABEL=${TAG}-${OS_ID}

rm -rf target
echo "Starting release build for ${LABEL}"

if [ ${OS_ID} = "linux" ]
then
  echo "Building statically linked linux binary"
  docker run --rm -v $(pwd):/home/rust/src -w /home/rust/src ekidd/rust-musl-builder \
    sh -c 'sudo chown -R rust:rust . && cargo build --release && cp target/x86_64-unknown-linux-musl/release/floki .'
  sudo chown -R $(id -u):$(id -g) .
  tar -cvzf floki-${LABEL}.tar.gz floki

else
  echo "Building release binary"
  cargo build --release
  zip -j floki-${LABEL}.zip target/release/floki
fi

echo "Release build complete"

#!/bin/sh

set -x

LABEL=${TRAVIS_TAG}-${TRAVIS_OS_NAME}

rm -rf target
echo "Starting release build for ${LABEL}"

if [ ${TRAVIS_OS_NAME} = "linux" ]
then
  echo "Building statically linked linux binary"
  docker run --rm -v $(pwd):/home/rust/src -w /home/rust/src ekidd/rust-musl-builder \
    sh -c 'sudo chown -R rust:rust . && cargo clippy --release && cargo build --release && cp target/x86_64-unknown-linux-musl/release/floki .'
  sudo chown -R travis:travis .
  tar -cvzf floki-${LABEL}.tar.gz floki

else
  echo "Building release binary"
  cargo build --release
  zip -j floki-${LABEL}.zip target/release/floki

fi

echo "Release build complete"

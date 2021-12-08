#!/usr/bin/env bash

echo "Announce changes to the world!"

VERSION=$(tomlq -r '.package.version' Cargo.toml)
echo "Version: $VERSION"

docker run \
   --rm \
  -v $PWD:/floki \
  metaswitch/announcer:3.0.2 \
    announce \
    --webhook $ANNOUNCE_HOOK \
    --target teams \
    --changelogversion $VERSION \
    --changelogfile /floki/CHANGELOG.md \
    --projectname floki \
    --iconemoji ship

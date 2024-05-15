#!/usr/bin/env bash

echo "Announce changes to the world!"

VERSION=$(cargo get package.version)
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

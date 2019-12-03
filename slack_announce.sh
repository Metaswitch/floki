#!/usr/bin/env bash

echo "Announce changes to the world!"

VERSION=$(tomlq -r '.package.version' Cargo.toml)
echo "Version: $VERSION"

# Send announcement to a standalone test Slack channel.
docker run \
   --rm \
  -v $PWD:/floki \
  metaswitch/announcer:2.3.0 \
    announce \
    --slackhook $ANNOUNCE_HOOK \
    --changelogversion $VERSION \
    --changelogfile /floki/CHANGELOG.md \
    --projectname floki \
    --username travis-announcer \
    --iconemoji ship

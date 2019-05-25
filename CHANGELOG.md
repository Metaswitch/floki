# Changelog

All notable changes to this project will be documented in this file.

This file's format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/). The
version number is tracked in the file `VERSION`.

## Unreleased
### Changed
### Added

## [0.1.0] - 2019-05-26
### Changed
- Remove `forward_tmux_socket` - BREAKING
- Remove `--pull` switch - BREAKING
- Remove pull specifications from configuration file - BREAKING
- Refactor to collect environment at start of day - PATCH
- Only mount the ssh_agent socket file - BREAKING
- Start working in the mount_pwd path - BREAKING
- Rename mount_pwd to mount - BREAKING
- Enforce reproducibility (override with `--local`) - BREAKING
- Move from `trim_right` to `trim_end` - PATCH
- (Refactor) Simplify addition of environment variables to docker run - PATCH
- Refactor - PATCH
- Add Travis CI file - PATCH
- Use 2018 edition of rust. - PATCH
- Update quicli to 0.4 - PATCH
- Deploy to GitHub - PATCH
- Make `sh` the default shell - BREAKING

### Added
- Make `pull` a subcommand of `floki` - MINOR

## [0.0.20] - 2019-02-12
### Changed

### Added
- Expose host user id as FLOKI_HOST_UID - MINOR
- Expose host user id as FLOKI_HOST_GID - MINOR
- Allow inner and outer shells to be specified - MINOR

## [0.0.19] - 2018-10-23
### Changed

- Exit if an `init` command fails (as opposed to carrying on) - BREAKING
- Make sure `floki` detects docker errors properly - BUGFIX
- Non-zero exit code on error - BUGFIX

## [0.0.18] - 2018-10-05
### Changed
- Make `floki run` work properly with subcommand switches - BUGFIX
- Make sure floki errors if `docker build` fails - BUGFIX

## [0.0.17] - 2018-10-02
### Added
- Package floki in an RPM - PATCH
- Add `floki run` subcommand - PATCH

## [0.0.16] - 2018-09-10
### Changed
- Wrapped common docker errors to make them clearer - PATCH

## [0.0.15] - 2018-08-08
### Changed
- Only kill `dind` container if we launched it - BUGFIX

## [0.0.14] - 2018-08-08
### Added
- --pull switch to update images - PATCH
### Fixed
- Fixup docker-in-docker to allow bind mounts - PATCH

## [0.0.13] - 2018-08-06
### Added
- docker-in-docker support - PATCH
- Add ability to forward current user - PATCH

## [0.0.12] - 2018-07-31
### Changed
- Made tmux socket forwarding permissive (doesn't fail if not found) - PATCH

## [0.0.11] - 2018-07-31
### Changed
- Build spec now requires the name as a subkey of build - BREAKING
- forward_tmux_session -> forward_tmux_socket - BREAKING

### Added
- Rewrite in Rust - PATCH
- Sphinx docs - PATCH

## [0.0.10] - 2018-07-25
### Added
- Allow custom docker switches - PATCH
- Configurable pull policy - PATCH

## [0.0.9] - 2018-07-12
### Added
- Add a version switch - PATCH

### Changed
- Make docker not use sudo - PATCH

## [0.0.8] - 2018-07-11
### Changed
- Empty init defaults to no commands - BUGFIX
- Make image specification mandatory - PATCH

## [0.0.7] - 2018-07-10
### Changed
- Change how we specify an image to build - PATCH

## [0.0.6] - 2018-07-10
### Added
- Add option to forward tmux socket - PATCH
- Add basic configuration validation - PATCH
- Added ability to specify shell - PATCH
- Add BSD style help switch - PATCH

## [0.0.5] - 2018-07-03
### Added
- Config file now command line parameter.  Default still `./floki.yaml`

## [0.0.4] - 2018-04-06
### Changed
- Allow build container to originate from Dockerfile - PATCH

## [0.0.3] - 2018-04-06
### Changed
- Rename to ssh-agent forwarding field - PATCH

## [0.0.2] - 2018-04-06
### Changed
- Rename to floki to prevent conflicts on pypi - PATCH

## [0.0.1] - 2018-04-06
### Added
- Initial primitive version
### Changed

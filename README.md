# floki

Floki was a boatbuilder. Floki now helps you manage interactive containers for building software.

## What is floki?

Docker and kubernetes are great ways to run software, and it is often convenient to use the same containers interactively to get a repeatable and complete build environment. However, using these containers for development is not always straightforward.

`floki` aims to improve the human interface for launching and using interactive docker containers. Instead of remembering or constructing complicated `docker run` commands, or writing custom scripts to launch docker containers, `floki` lets you specify what you want from your docker container in a configuration file. You can then get your environment just by running `floki`. It doesn't replace docker or kubernetes, its an addition to try and improve the human interface for working on a codebase.

This has several advantages over the usual approaches (custom scripts, or, more commonly, leaving it to the user to figure out)

- an immediate build environment
- easier to share and on-board new developers
- a consistent and uniform interface to get a working environment

## Documentation

For installation, and basic usage, see [getting started](https://metaswitch.github.io/floki/documentation/getting-started/).

Full documentation can be found [here](https://metaswitch.github.io/floki/).

### Quickstart

Suppose we want a build environment based on `alpine:latest` with a C compiler, and `clang` tools. Suppose we want to also have SSH credentials available from the host, so we can, for example, authenticate with a private git server.

First create your `Dockerfile`:

```
FROM alpine:latest

RUN apk update && apk add alpine-sdk clang openssh
```

and then add a file called `floki.yaml` to the root of your codebase:


```
image:
  build:
    name: hello-floki

forward_ssh_agent: true
```

Now run `floki`. You should see the docker container be built, and you will be dropped into a shell. If you had an `ssh-agent` running on the host before running `floki`, you can run `ssh-add -l` and you should see the same keys loaded as you had on the host.

## Install

### From pre-built binaries

Precompiled binaries can be downloaded from the releases page (for linux and OSX).

To obtain `curl` and extract the latest linux binary directly in your shell, run

```
$ curl -L https://github.com/Metaswitch/floki/releases/download/0.4.3/floki-0.4.3-linux.tar.gz | tar xzvf -
```

You should be able to run `floki` from your working directory:

```
$ ./floki --version
floki 0.4.3
```

Move it onto your path to run it from anywhere. E.g.

```
# mv floki /usr/local/bin/
```

Enjoy!

### From cargo

`floki` can also be installed directly from `cargo`.

```
$ cargo install floki
```

## Prerequisites

It's recommended you add your user to the `docker` group:

```
sudo usermod -a -G docker USERNAME
```

and logout and in again to pick up the changes.

Alternatively you can run `floki` with `sudo -E floki`.

## Basic usage

`floki` allows you to launch interactive containers with your working directory mounted, and to configure those containers for interactive use.

Provide a `floki.yaml` in the root directory of your source code, in the following form:

```yaml
image: $IMAGE_NAME
init:
  - echo "Hello, there"
```

or to use a local `Dockerfile`

```yaml
image:
  build:
    name: name_of_resultant_image
    dockerfile: Dockerfile.build
init:
  - echo "Hello, there"
```

and run `floki` in that directory. You should be dropped into a shell with the init commands run, and inside a directory where your host working directory has been mounted.

## Handy features

- Forwarding of `ssh-agent` (useful for authenticating with remote private git servers to pull private dependencies)
- Docker-in-docker support
- Forwarding of host user information (allows non-root users to be added and used).

## Contributing

Contributors will need to sign their commits to acknowledge the [DCO](DCO)

## TODO

See issues.

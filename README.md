# floki

Floki was a boatbuilder. Floki now helps you launch build containers.

## What is floki?

`floki` aims to improve the human interface for launching and using interactive docker containers. Instead of remembering or constructing complicated `docker run` commands, or writing custom scripts to launch docker containers, `floki` lets you specify what you want from your docker container in a configuration file. You can then get your environment just by running `floki`.

This has several advantages over the usual approaches (custom scripts, or, more commonly, leaving it to the user to figure out)

- easier to use
- easier to share
- if adopted across multiple projects, it provides a consistent and uniform interface to get a working environment

## Documentation

For installation, and basic usage, see [getting started](https://metaswitch.github.io/floki/documentation/getting-started/).

Full documentation can be found [here](https://metaswitch.github.io/floki/).

## Install

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

Contributors will need to sign there commits to acknowledge the [DCO](DCO) 

## TODO

See issues.

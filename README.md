# floki

Floki was a boatbuilder. Floki now helps you launch build containers.

## What is floki?

`floki` aims to improve the human interface for launching and using interactive docker containers. Instead of remembering or constructing complicated `docker run` commands, or writing custom scripts to launch docker containers, `floki` lets you specify what you want from your docker container in a configuration file. You can then get your environment just by running `floki`.

This has several advantages over the usual approaches (custom scripts, or, more commonly, leaving it to the user to figure out)

- easier to use
- easier to share
- if adopted across multiple projects, it provides a consistent and uniform interface to get a working environment

## Documentation

## Install

## Prerequisites

It's recommended you add your user to the `docker` group:

```
sudo usermod -a -G docker USERNAME
```

and logout and in again to pick up the changes.

Alternatively you can run `floki` with `sudo -E floki`.

## Usage

Provide a `floki.yaml` in the root directory of your source code, in the following form:

```yaml
image: $IMAGE_NAME
forward_ssh_agent: true
init:
  - cd /src
```

or to use a local `Dockerfile`

```yaml
image:
  build:
    name: name_of_resultant_image
    dockerfile: Dockerfile.build
forward_ssh_agent: true
init:
  - cd /src
```

and run `floki` in that directory. You should be dropped into a shell with the init commands run, and your source code mounted at `/src`.

## Contributing

Contributors will need to sign there commits to acknowledge the [DCO](DCO) 

## TODO

See issues.

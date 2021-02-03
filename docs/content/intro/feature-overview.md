---
title: "Feature Overview"
date: 2019-04-06T20:22:44+01:00
draft: false
---

`floki` aims to provide reproducible and shareable build tooling. It does this by helping you run docker containers interactively from a declarative yaml file.

The ideal workflow is

- clone the source repository
- run `floki`
- get to work

`floki` has a number of features to help achieve this. The following outlines these features.

# Container images

`floki` offers a couple of ways to configure the container image to use.

## Prebuilt images

Using a prebuilt image (e.g. one from dockerhub or a docker registry) is as simple as providing its name as a top-level key in `floki.yaml`:

```yaml
image: debian:sid
```

`floki` will use docker to pull this image if you need it.

Custom registries can be used by configuring `docker` to use these registries. `floki` defers to `docker` to locate and pull images.

## Build an image

`floki` can use an image built from a `Dockerfile` in source tree. It's easiest to see an example of `floki.yaml` to see how to configure this.

```yaml
image:
  build:
    name: foo                    # Will build the image with name foo:floki
    dockerfile: Dockerfile.foo   # Relative location in source tree; defaults to Dockerfile
    context: .                   # Defaults to .
    target: builder              # Target to use, for multi-stage dockerfiles (optional)
```

## Referencing a key in another yaml file
`floki` can use an image by reference to another yaml file. This can help keep local development environments synced with a CI environment.

```yaml
image:
  yaml:
    file: .gitlab-ci.yaml
    key: variables.RUST-IMAGE
```

## Updating an image

`floki pull` forces a pull of the container specified in `image`. While it is better to version images properly, this can be used when tracking a `latest` tag, or similar.

# Setting the shell

Different containers require different shells, so `floki` allows you to configure this. Sometimes you will want a different shell to run the `init` commands to the shell presented to the user, and so `floki` also allows you to set an outer (used for `init`) and inner (used by the user) shell.

The default shell is `sh`.

## Single shell

A shell can be set for a container using the top-level `shell` key:

```yaml
image: alpine:latest
shell: sh
```

## Inner and outer shell

A different shell can be used for initialization and the interactive shell provided to the user.

```yaml
image: alpine:latest
shell:
  inner: bash
  outer: sh
init:
  - apk update && apk install bash
```

A useful use case here is if you want to run the container with the same user as on the host. `floki` exposes the user id and user group id in environment variables, so you can add a user to the running container and switch to the new user in the inner shell:

```yaml
image: foo:latest
shell:
  inner: bash
  outer: switch_user
init:
  - add_new_user $FLOKI_HOST_UID $FLOKI_HOST_GID
```

The commands to make the above work depend on the container you are running. `floki` just provides the tools to allow you to make it happen.

# Docker-in-docker

Docker-in-docker (`dind`) can be enabled by setting the top-level `dind` key to `true`.

```yaml
image: foo:bar
dind: true
```

Note that the docker CLI tools are still required in the container, and the docker host is a linked container, with the working directory mounted in the same place as the interactive container.

The precise `dind` image can also be set

```yaml
dind:
  image: docker:stable-dind
```

This helps properly pin and version the docker-in-docker container.

# Floki volumes

`floki` has the ability to use volumes for caching build artifacts between runs of the container (amongst other things). Volumes can be configured in `floki.yaml`:

```yaml
volumes:
  cargo-registry:
    mount: /home/rust/.cargo/registry
```

The key names the volume (it can be any valid yaml name), while the `mount` key specifies where the volume will be mounted inside the `floki` container.

It's also possible to share volumes across different `floki.yaml`s. For example, you may want to share a `cargo` registry across all Rust build containers. These shared volumes are identified by the name given to the volume.

```yaml
volumes:
  cargo-registry:
    shared: true
    mount: /home/rust/.cargo/registry
```

`floki` creates directories on the host to back these volumes in `~/.floki/volumes`. Non-shared volumes are given names unique to the source directory.

# Environment forwarding

## User details

`floki` captures the host user details in environment variables, and forwards these into the running container.

* `FLOKI_HOST_UID` is set to the host user's user id (the output of `id -u`)
* `FLOKI_HOST_GID` is set to the host user's group id (the output of `id -g`)

These can be used to configure users in the container dynamically. This can be a little fiddly, especially if the container already uses a non-root user with the same id as the host user.

## Host working directory

The host path to the mounted directory is forwarded into the `floki` container as an environment variable, `FLOKI_HOST_MOUNTDIR`.

You can set where this directory is mounted in the container using the `mount` key in `floki.yaml`.

## SSH agent

Sometimes it is useful to be able to pull dependencies from source code management servers for builds. To make this easier to do in an automated fashion, `floki` can forward and `ssh-agent` socket into the container, and expose its path through `SSH_AUTH_SOCK`.

```yaml
forward_ssh_agent: true
```

You will need to have an `ssh-agent` running on the host before launching `floki`.

# Sandboxed commands with floki run

`floki` also allows single commands to be run, rather than dropping into an interactive shell.

```shell
$ floki run ls
floki.yaml
```

Note that if you have configured an inner shell, the command will run within the inner shell.


# Escaping with `docker_switches`

`floki` also allows you to pass additional switches to the underlying docker command, for example to forward port `8080` to the host.

```yaml
image: debian:sid
docker_switches:
  - -p
  - 8080:8080
init:
  - echo "Welcome to your server container!"
```

Note that use of `docker_switches` may reduce the reproducibility and shareability of your `floki.yaml` (for instance it could be used to mount a volume with a specific host path that works on no other machines).

Nonetheless, it is useful to be able to add arbitrary switches in a pinch, just to be able to get something working.
If there are things you can add with `docker_switches` which are reproducible and shareable, please raise a feature request, or go ahead and implement it yourself!

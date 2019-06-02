---
title: "Getting Started"
date: 2019-04-03T23:01:31+01:00
draft: false
---

## Installation

Precompiled binaries can be downloaded from the releases page (for linux (statically linked) and OSX).

To obtain `curl` and extract the latest linux binary directly in your shell, run

```
$ curl -L https://github.com/Metaswitch/floki/releases/download/0.1.0/floki-0.1.0-linux.tar.gz | tar xzvf -
```

You should be able to run `floki` from your working directory:

```
$ ./floki --version
floki 0.1.0
```

Move it onto your path to run it from anywhere. E.g.

```
# mv floki /usr/local/bin/
```

Enjoy!

## Getting started

Write a basic configuration file, and name it `floki.yaml`.

```
image: debian:latest
init:
  - echo "Welcome to your first floki container!"
```

In the same directory, run

```
floki
```

A container will launch with the working directory mounted, and the container shell located there.

## Using a different configuration file

You can use a different configuration file with `floki` by telling it to use a different file from the command line. For example, if you have another configuration in `config.yaml`, you can run `floki` with

```
floki -c config.yaml
```

### Features you may want to look at next

- Forwarding of `ssh-agent` (useful for authenticating with remote private git servers to pull private dependencies)
- Docker-in-docker support
- Forwarding of host user information (allows non-root users to be added and used).

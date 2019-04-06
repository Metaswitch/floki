---
title: "Getting Started"
date: 2019-04-03T23:01:31+01:00
draft: false
---

## Installation

Statically linked binaries are built as part of CI for tags, and can be copied directly onto your system.

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


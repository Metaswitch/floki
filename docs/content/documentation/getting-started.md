---
title: "Getting Started"
date: 2019-04-03T23:01:31+01:00
draft: false
---

## Installation

Precompiled binaries for linux and OSX can be downloaded from the [releases](https://github.com/Metaswitch/floki/releases) page.

For example, to obtain the latest binary with `curl` and extract it, run

```
$ curl -L https://github.com/Metaswitch/floki/releases/download/0.6.1/floki-0.6.1-linux.tar.gz | tar xzvf -
```

in a shell. You should now be able to run `floki` from your working directory:

```
$ ./floki --version
floki 0.6.1
```

Copy this into your path to run it without needing to specify the path absolutely. E.g.

```
# mv floki /usr/local/bin/
```

Shell completions can be added to your existing shell session with

```
source <(floki completion <shell>)
```

See `floki completion --help` for a list of available `<shell>`s. Add this command to your shell's rc file to get completions in all new shell sessions.

Enjoy!

## Getting started

`floki` is configured using a configuration file typically placed in the root of your codebase. As a basic example, write a basic configuration file, and name it `floki.yaml`.

```
image: debian:latest
init:
  - echo "Welcome to your first floki container!"
```

Now, in the same directory, run

```
floki
```

A container will launch with the working directory mounted as your working directory. Verify this by running `ls`:

```
$ ls
...  floki.yaml  ...
```

In general, invoking `floki` in any child directory of this root directory will launch a container with:
- The directory containing `floki.yaml` mounted;
- The container shell located in the guest directory corresponding to the child.

## Using a different configuration file

You can use a different configuration file with `floki` by telling it to use a different file from the command line. For example, if you have another configuration in `config.yaml`, you can run `floki` with

```
floki -c config.yaml
```

Note that, in contrast to invoking `floki` without the `-c` flag, this will always mount the current working directory.

### Features you may want to look at next

- Forwarding of `ssh-agent` (useful for authenticating with remote private git servers to pull private dependencies)
- Docker-in-docker support
- Forwarding of host user information (allows non-root users to be added and used).
- `floki` volumes for setting up cross-session build caches.

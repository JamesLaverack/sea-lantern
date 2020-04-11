# Sea Lantern

![Project Status](https://img.shields.io/badge/project_status-alpha-red)
![Rust Build](https://github.com/JamesLaverack/sea-lantern/workflows/Rust%20Build/badge.svg)

An Operator for Minecraft on Kubernetes.

## Motivation

There are many reasons for wanting to run Minecraft in Kubernetes:

* You want to take advantage of spare capacity on an existing cluster.
* You want to run multiple servers on the same hardware.
* You want to run Minecraft and other applications on the same hardware.

However, if you only want Minecraft and you don't know a lot about Kubernetes to begin with, then it's likely to be a
steep learning curve.

## Current Status

* The [Management API](src/bin/management_api.rs) exists as a standalone component that will communicate with a
  Minecraft server that has enabled [RCON](https://wiki.vg/RCON). It fronts the RCON API into
  [a gRPC one](api/proto/management/management.proto).

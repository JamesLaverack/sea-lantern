# Sea Lantern

An Operator for Minecraft on Kubernetes

# Architecture

## Init Containers

- [JAR Fetcher](src/bin/jar_fetcher.rs) fetches the Minecraft server JAR from an S3-compatible store.
- [Mod Fetcher](src/bin/mod_fetcher.rs) fetches mods for Forge and Spigot servers.
- [EULA Writer](src/bin/eula_writer.rs) writes the eula.txt file for the server depending on the spec.eula field.

## Main Container & sidecars

- [Runtime](src/bin/runtime.rs) runs in the main container and executes Java to run the Minecraft server. STDIN and STDOUT are
  captured from the child process and forwarded to anything that connects on a local UNIX socket. This allows sidecar
  containers to use the Minecraft console API.
- [Management API](src/bin/management_api.rs) runs as a sidecar. It connects to the UNIX socket exposed by the runtime and provides a gRPC
  API equivilent of the Minecraft console API. It also provides gRPC methods to download the data directory of the
  server.
- [Backup Agent](src/bin/backup_agent.rs) runs as a sidecar, and uses the management API to perform backups, uploading them to an S3-compatable store.

## Other components
- [Operator](src/bin/operator.rs) runs a Deployment in the `sea-lantern-system` Namespace and orchastrates everything.
- [JAR Bakery](src/bin/jar_bakery.rs) runs as a Job and 'bakes' Minecraft server JARs into the S3 store. Including Forge
  and Spigot.
- [Map Renderer](src/bin/map_renderer.rs) runs as Job and executes [Minecraft Overviewer](https://github.com/overviewer/Minecraft-Overviewer) on save files.
- [NGINX](https://www.nginx.com) serving the rendered map from MinIO.

# Supported Server Versions

- Vanilla
- Spigot
- Forge
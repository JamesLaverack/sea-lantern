# Architecture

- The Operator itself runs as Deployment in the `sea-lantern-system` Namespace and orchestrates
  everything.
- The `MinecraftServer`, `MinecraftServerVersion`, and `MinecraftRollingServerVersion` custom resource definitions.

## Per Server Version

For each `MinecraftServerVersion`, the operator runs a Job that will build a container image for that version and
upload it to a given container registry. The `status.image` field of the resource will be set to the name of the built
image.

A rolling server version pins a spesific distribution (e.g., paper, spigot, etc.) and minecraft version (e.g., 1.15.2)
but will auto-generate new `MinecraftServerVersion` resources each time the underlying distribution upgrades. 

Servers can set either `spec.serverVersionName` or `spec.rollingServerVersionName`.

## Per-Server

For each `MinecraftServer` you get, in the server's namespace:
- Server controller runs as a Deployment and manages all resources for that
  server. 
- [Management API](src/bin/management_api.rs) runs as a Deployment and is a kind of "proxy" or "gateway" that connects
  to the server's RCON API and provides a gRPC API for external use. Authentication is provided using Kuberentes
  service accounts.
- A Pod to be the minecraft server itself:
  - The EULA Writer runs as an init container and writes the eula.txt file for the server
    depending on the `spec.eula` field of the `MinecraftServer` resource.
  - Minecraft runs in a Container by itself, just the server JAR and a Java runtime. The only requirement is that RCONs
    is enabled.
  - Config Updater runs as a sidecar, and continually pushes updates to the `server.properties` file.

# Supported Server Versions

- Vanilla
- Spigot
- Forge
- Paper
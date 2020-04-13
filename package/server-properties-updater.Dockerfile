FROM gcr.io/distroless/cc

COPY target/release/server_properties_updater /bin/server-properties-updater

CMD ["/bin/server-properties-updater"]

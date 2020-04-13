FROM gcr.io/distroless/cc

COPY target/release/eula_writer /bin/eula-writer

CMD ["/bin/eula-writer"]

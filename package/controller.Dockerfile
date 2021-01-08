FROM gcr.io/distroless/cc

COPY target/release/controller /bin/controller

CMD ["/bin/controller"]

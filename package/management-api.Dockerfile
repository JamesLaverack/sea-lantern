FROM gcr.io/distroless/cc

COPY target/release/management_api /bin/management-api

CMD ["/bin/management-api"]

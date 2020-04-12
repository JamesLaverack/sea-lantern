FROM gcr.io/distroless/base

COPY target/release/management_api /bin/management-api

CMD ["/bin/management-api"]

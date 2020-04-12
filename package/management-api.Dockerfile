FROM gcr.io/distroless/base:nonroot

COPY target/release/management_api /bin/management-api

CMD ["/bin/management-api"]

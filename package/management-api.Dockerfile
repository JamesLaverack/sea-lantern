FROM rust:1.42 as build

COPY ./ ./

RUN cargo build --release --bin management_api

FROM gcr.io/distroless/base

COPY --from=builder target/release/management_api /bin/management-api

CMD ["/bin/management-api"]

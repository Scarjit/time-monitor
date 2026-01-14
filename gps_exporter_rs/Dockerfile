FROM docker.io/rust:alpine as build

WORKDIR /build
COPY . .

RUN cargo build --release

FROM scratch
COPY --from=build /build/target/release/gps_exporter_rs .

ENTRYPOINT ["/gps_exporter_rs"]
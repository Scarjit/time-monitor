FROM alpine:latest AS base

ARG TARGETARCH

RUN --mount=type=bind,source=./target/aarch64-unknown-linux-musl/release,target=/mnt/arm \
    --mount=type=bind,source=./target/x86_64-unknown-linux-musl/release,target=/mnt/x86 \
    if [ "$TARGETARCH" = "arm64" ] && [ -f /mnt/arm/gps_exporter_rs ]; then \
      cp /mnt/arm/gps_exporter_rs /usr/local/bin/gps_exporter_rs; \
    elif [ "$TARGETARCH" != "arm64" ] && [ -f /mnt/x86/gps_exporter_rs ]; then \
      cp /mnt/x86/gps_exporter_rs /usr/local/bin/gps_exporter_rs; \
    else \
      echo "Error: No service binary found for architecture $TARGETARCH!" >&2; \
      ls -R /mnt >&2; \
      exit 1; \
    fi && \
    chmod +x /usr/local/bin/gps_exporter_rs

FROM base AS default

ENTRYPOINT ["/usr/local/bin/gps_exporter_rs"]
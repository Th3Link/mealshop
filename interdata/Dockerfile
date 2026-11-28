# Build from the repository root: docker build -f interdata/Dockerfile .
FROM rust:1.100-alpine@sha256:8d0e82abfdf9a38071e07cabc5afba4ddbe077631a80e4e742a760c38beadc33 AS build
RUN apk add --no-cache musl-dev
WORKDIR /build
# The root package is a workspace member, so its manifest and sources must be present.
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY interdata/ interdata/
RUN cargo build --release --locked --package mealshop-interdata

FROM alpine:3.24.4@sha256:f59abe656b692d901e184be3445e084786602108599d2a51d229c0f5a48c71bf
RUN addgroup -S -g 10001 interdata \
    && adduser -S -u 10001 -G interdata interdata \
    && install -d -o interdata -g interdata /data
COPY --from=build /build/target/release/mealshop-interdata /usr/local/bin/mealshop-interdata
USER 10001:10001
ENV INTERDATA_LISTEN_ADDR=0.0.0.0:3100 \
    INTERDATA_DATA_DIR=/data
EXPOSE 3100
VOLUME /data
HEALTHCHECK --interval=30s --timeout=5s --retries=3 --start-period=5s \
  CMD wget -q -O /dev/null http://127.0.0.1:3100/health
ENTRYPOINT ["/usr/local/bin/mealshop-interdata"]

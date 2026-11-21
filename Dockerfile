FROM node:26-alpine@sha256:6bd9ab43c0ec0190ca7b2d193906cb8b7edf6e7f4afa43ddeccd6452710bffb5 AS web
WORKDIR /build/clients/nextcloud
COPY clients/nextcloud/package*.json ./
COPY scripts/npm_toolchain.mjs /build/scripts/npm_toolchain.mjs
RUN npm_spec="$(node ../../scripts/npm_toolchain.mjs)" \
    && npm install --global "$npm_spec" --ignore-scripts --no-audit --no-fund
RUN npm ci --ignore-scripts --no-audit --no-fund
COPY clients/nextcloud/ ./
COPY data/allergens.json data/intolerance-triggers.json /build/data/
RUN npm run build

FROM rust:1.100-alpine@sha256:8d0e82abfdf9a38071e07cabc5afba4ddbe077631a80e4e742a760c38beadc33 AS server
RUN apk add --no-cache musl-dev build-base
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY data/ data/
COPY appinfo/ appinfo/
RUN cargo build --release --locked

FROM alpine:3.24.4@sha256:f59abe656b692d901e184be3445e084786602108599d2a51d229c0f5a48c71bf
RUN apk add --no-cache bash curl frp ca-certificates
COPY --from=server /build/target/release/mealshop-server /usr/local/bin/mealshop-server
COPY --from=web /build/clients/nextcloud/dist /opt/mealshop/web
COPY appinfo/ /appinfo/
COPY deploy/nextcloud/start.sh /start.sh
RUN chmod 755 /start.sh
ENV MEALSHOP_ASSET_DIR=/opt/mealshop/web
HEALTHCHECK --interval=30s --timeout=5s --retries=3 --start-period=10s --start-interval=1s \
  CMD sh -c 'curl -sf --unix-socket /tmp/exapp.sock http://localhost/heartbeat || curl -sf "http://127.0.0.1:${APP_PORT}/heartbeat"'
ENTRYPOINT ["/start.sh", "/usr/local/bin/mealshop-server"]

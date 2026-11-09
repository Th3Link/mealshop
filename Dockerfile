FROM node:24-alpine@sha256:a34ab6d7f6f4813f4d0c5469d74685604b962fbdbf5793a7bc7b4998ecb867c8 AS web
WORKDIR /build/clients/nextcloud
COPY clients/nextcloud/package*.json ./
RUN npm install --global npm@12.2.2 --ignore-scripts --no-audit --no-fund
RUN npm ci --ignore-scripts --no-audit --no-fund
COPY clients/nextcloud/ ./
COPY data/allergens.json data/intolerance-triggers.json /build/data/
RUN npm run build

FROM rust:1.85-alpine@sha256:4333721398de61f53ccbe53b0b855bcc4bb49e55828e8f652d7a8ac33dd0c118 AS server
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

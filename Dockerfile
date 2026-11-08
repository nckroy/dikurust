# syntax=docker/dockerfile:1

# ---- build ---------------------------------------------------------------
# Matches the channel in rust-toolchain.toml, so rustup has nothing to fetch.
FROM rust:1.96.1-slim-trixie AS build
WORKDIR /src

# Build the (empty) dependency graph first so source edits reuse this layer.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
 && cargo build --release --locked \
 && rm -rf src target/release/dmserver target/release/deps/dmserver-* target/release/.fingerprint/dikumud-*

COPY src ./src
COPY web ./web
RUN cargo build --release --locked

# ---- runtime -------------------------------------------------------------
FROM debian:trixie-slim

# tini runs as PID 1 so `docker stop` (SIGTERM) ends the server at once;
# dmserver has no signal handler and PID 1 ignores unhandled SIGTERM.
RUN apt-get update \
 && apt-get install -y --no-install-recommends tini \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --home-dir /var/lib/dikumud --shell /usr/sbin/nologin dikumud \
 && install -d -o dikumud -g dikumud /var/lib/dikumud /var/lib/dikumud/plrs

# Pristine game data. The entrypoint copies it into the data volume on start.
COPY lib /usr/share/dikumud/lib
COPY doc/license.doc /usr/share/doc/dikumud/license.doc
COPY --from=build /src/target/release/dmserver /usr/local/bin/dmserver
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

USER dikumud
WORKDIR /var/lib/dikumud
VOLUME ["/var/lib/dikumud"]
EXPOSE 4000

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s \
  CMD ["bash", "-c", ":> /dev/tcp/127.0.0.1/4000"]

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/docker-entrypoint.sh"]
# -x: the container is meant to sit behind a reverse proxy on the same host
# (Caddy on diku.earth, or whatever fronts compose.yaml's loopback port), so
# take players' addresses from its X-Forwarded-For.
CMD ["-x", "-b", "0.0.0.0", "4000"]

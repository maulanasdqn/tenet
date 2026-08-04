FROM rust:1-bookworm AS builder

ARG SERVICE
ENV CARGO_TERM_COLOR=never

WORKDIR /build
COPY . .
ARG CARGO_BUILD_JOBS

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/build/target,sharing=locked \
    cargo build --release --locked ${CARGO_BUILD_JOBS:+--jobs ${CARGO_BUILD_JOBS}} -p "tenet-${SERVICE}" \
 && cp "target/release/tenet-${SERVICE}" /build/entrypoint

FROM debian:bookworm-slim AS runtime

ARG SERVICE

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && if [ "${SERVICE}" = "analyzer" ]; then \
      apt-get install -y --no-install-recommends \
        chromium \
        fonts-liberation \
        fonts-noto-color-emoji \
        libnss3 \
        libxss1 \
        dumb-init ; \
    fi \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --no-create-home tenet

RUN mkdir -p /home/tenet && chown 10001:10001 /home/tenet

ENV CHROME_BIN=/usr/bin/chromium \
    BROWSER_NO_SANDBOX=1 \
    HOME=/home/tenet \
    XDG_CONFIG_HOME=/home/tenet \
    XDG_CACHE_HOME=/home/tenet

COPY --from=builder /build/entrypoint /usr/local/bin/entrypoint

USER 10001
ENTRYPOINT ["/usr/local/bin/entrypoint"]

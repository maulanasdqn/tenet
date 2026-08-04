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

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --no-create-home tenet

COPY --from=builder /build/entrypoint /usr/local/bin/entrypoint

USER 10001
ENTRYPOINT ["/usr/local/bin/entrypoint"]

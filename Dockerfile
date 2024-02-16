# syntax=docker/dockerfile:1.2

ARG RUST_VERSION=1.70

FROM rust:${RUST_VERSION} as builder

ENV SQLX_OFFLINE=true

RUN --mount=type=bind,source=src,target=/src \
    --mount=type=bind,source=migrations,target=/migrations \
    --mount=type=bind,source=Cargo.toml,target=/Cargo.toml \
    --mount=type=bind,source=.sqlx,target=/.sqlx \
    --mount=type=bind,source=Cargo.lock,target=/Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    set -e && cargo build --locked --release && cp ./target/release/rinha-24 /bin/rinha-24

FROM debian:bullseye as final

COPY --from=builder /bin/rinha-24 /bin/rinha-24

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    rinhaosuser
USER rinhaosuser

EXPOSE 8000

CMD ["/bin/rinha-24"]
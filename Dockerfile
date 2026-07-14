FROM rust:bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --locked --release --bin api --bin cli

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 fundingflow

COPY --from=builder /app/target/release/api /usr/local/bin/api
COPY --from=builder /app/target/release/cli /usr/local/bin/cli

USER fundingflow
ENV PORT=3001 \
    RUST_LOG=info
EXPOSE 3001

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:3001/health >/dev/null || exit 1

CMD ["api"]
